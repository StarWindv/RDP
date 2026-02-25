//! Parser for shell script AST generation

use std::iter::Peekable;
use std::vec::IntoIter;

use crate::modules::lexer::Lexer;
use crate::modules::tokens::{Token, TokenType};
use crate::modules::ast::{AstNode, CommandSeparator, ParseError, RedirectType};

/// Parser for shell scripts
pub struct Parser<'a> {
    input: &'a str,
    tokens: Peekable<IntoIter<Token>>,
    current_token: Option<Token>,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given input
    pub fn new(input: &'a str) -> Self {
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.collect::<Vec<_>>();
        let tokens_iter = tokens.into_iter().peekable();
        
        let mut parser = Self {
            input,
            tokens: tokens_iter,
            current_token: None,
        };
        
        parser.advance();
        parser
    }
    
    /// Parse the entire input into an AST
    pub fn parse(&mut self) -> Result<AstNode, ParseError> {
        println!("DEBUG PARSER: Starting parse");
        let result = self.parse_command_list();
        match &result {
            Ok(ast) => println!("DEBUG PARSER: Parse successful, AST type: {:?}", std::mem::discriminant(ast)),
            Err(e) => println!("DEBUG PARSER: Parse failed: {}", e),
        }
        result
    }
    
    /// Parse a command list (multiple commands separated by ; or newline)
    fn parse_command_list(&mut self) -> Result<AstNode, ParseError> {
        let mut commands = Vec::new();
        let mut separators = Vec::new();
        let mut tokens = Vec::new();
        
        // Parse first command
        let first_cmd = self.parse_command()?;
        commands.push(Box::new(first_cmd));
        
        // Parse additional commands with separators
        while let Some(token) = &self.current_token {
            match token.token_type {
                TokenType::Semicolon => {
                    tokens.push(token.clone());
                    separators.push(CommandSeparator::Semicolon);
                    self.advance();
                    
                    // Check for empty command (;; is valid in case statements)
                    if self.peek_token_type() == Some(&TokenType::Semicolon) {
                        // Double semicolon, continue
                        continue;
                    }
                    
                    // Parse next command
                    let cmd = self.parse_command()?;
                    commands.push(Box::new(cmd));
                }
                TokenType::Newline => {
                    tokens.push(token.clone());
                    separators.push(CommandSeparator::Newline);
                    self.advance();
                    
                    // Parse next command if available
                    if !self.is_at_end() {
                        let cmd = self.parse_command()?;
                        commands.push(Box::new(cmd));
                    }
                }
                TokenType::Ampersand => {
                    tokens.push(token.clone());
                    separators.push(CommandSeparator::Ampersand);
                    self.advance();
                    
                    // Parse next command if available
                    if !self.is_at_end() {
                        let cmd = self.parse_command()?;
                        commands.push(Box::new(cmd));
                    }
                }
                _ => break,
            }
        }
        
        if commands.len() == 1 {
            Ok(*commands.remove(0))
        } else {
            Ok(AstNode::CommandList {
                commands,
                separators,
                tokens,
            })
        }
    }
    
    /// Parse a command (could be simple command, pipeline, or compound command)
    fn parse_command(&mut self) -> Result<AstNode, ParseError> {
        // Check for compound commands
        if self.check_token_type(&TokenType::If) {
            return self.parse_if_statement();
        } else if self.check_token_type(&TokenType::While) {
            return self.parse_while_loop();
        } else if self.check_token_type(&TokenType::Until) {
            return self.parse_until_loop();
        } else if self.check_token_type(&TokenType::For) {
            return self.parse_for_loop();
        } else if self.check_token_type(&TokenType::LeftBrace) {
            return self.parse_compound_command();
        } else if self.check_token_type(&TokenType::Function) {
            return self.parse_function_definition();
        } else if self.check_token_type(&TokenType::LeftParen) {
            return self.parse_subshell();
        }
        
        // Parse logical OR expression (lowest precedence)
        self.parse_logical_or()
    }
    
    /// Parse logical OR expression: left || right
    fn parse_logical_or(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_logical_and()?;
        let mut tokens = Vec::new();
        
        while self.check_token_type(&TokenType::OrIf) {
            let token = self.current_token.clone().unwrap();
            tokens.push(token);
            self.advance();
            
            let right = self.parse_logical_and()?;
            
            left = AstNode::LogicalOr {
                left: Box::new(left),
                right: Box::new(right),
                tokens: tokens.clone(),
            };
        }
        
        Ok(left)
    }
    
    /// Parse logical AND expression: left && right
    fn parse_logical_and(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_pipeline()?;
        let mut tokens = Vec::new();
        
        while self.check_token_type(&TokenType::AndIf) {
            let token = self.current_token.clone().unwrap();
            tokens.push(token);
            self.advance();
            
            let right = self.parse_pipeline()?;
            
            left = AstNode::LogicalAnd {
                left: Box::new(left),
                right: Box::new(right),
                tokens: tokens.clone(),
            };
        }
        
        Ok(left)
    }
    
    /// Parse pipeline: command1 | command2
    fn parse_pipeline(&mut self) -> Result<AstNode, ParseError> {
        let mut commands = Vec::new();
        let mut tokens = Vec::new();
        
        // Parse first command
        commands.push(Box::new(self.parse_simple_command()?));
        
        // Parse additional commands in pipeline
        while self.check_token_type(&TokenType::Pipe) {
            let token = self.current_token.clone().unwrap();
            tokens.push(token);
            self.advance();
            
            commands.push(Box::new(self.parse_simple_command()?));
        }
        
        if commands.len() == 1 {
            Ok(*commands.remove(0))
        } else {
            Ok(AstNode::Pipeline {
                commands,
                tokens,
            })
        }
    }
    
    /// Parse a simple command (with optional redirections and background)
    fn parse_simple_command(&mut self) -> Result<AstNode, ParseError> {
        let mut assignments = Vec::new();
        let mut name = None;
        let mut args = Vec::new();
        let mut command_tokens = Vec::new();
        
        // Parse command name and arguments
        while let Some(token) = &self.current_token {
            match &token.token_type {
                TokenType::Word(word) | TokenType::Name(word) => {
                    if name.is_none() {
                        name = Some(word.clone());
                    } else {
                        args.push(word.clone());
                    }
                    command_tokens.push(token.clone());
                    self.advance();
                }
                TokenType::AssignmentWord(value) => {
                    if name.is_none() {
                        // This is a variable assignment before command name
                        assignments.push(AstNode::Assignment {
                            name: value.split('=').next().unwrap().to_string(),
                            value: value.splitn(2, '=').nth(1).unwrap_or("").to_string(),
                            token: token.clone(),
                        });
                        command_tokens.push(token.clone());
                        self.advance();
                    } else {
                        // This is an argument (e.g., export VAR=value)
                        args.push(value.clone());
                        command_tokens.push(token.clone());
                        self.advance();
                    }
                }
                TokenType::Question => {
                    // $? special variable
                    let question_token = token.clone();
                    command_tokens.push(question_token);
                    self.advance();
                    
                    // Create expanded argument
                    let expanded = "$?".to_string();
                    if name.is_none() {
                        name = Some(expanded);
                    } else {
                        args.push(expanded);
                    }
                }
                _ => {
                    // Unknown token type, break
                    break;
                }
            }
        }
        
        // If no command name was found, check if we have assignments only
        if name.is_none() && !assignments.is_empty() {
            // Just assignments, no command - create a compound command with assignments
            let commands: Vec<Box<AstNode>> = assignments.into_iter().map(|a| Box::new(a)).collect();
            return Ok(AstNode::CompoundCommand {
                commands,
                tokens: command_tokens,
            });
        }
        
        let name = name.ok_or_else(|| ParseError {
            message: "Expected command name".to_string(),
            token: Token::new(TokenType::Error("No command".to_string()), "".to_string(), 1, 1),
        })?;
        
        // Create simple command node
        let mut command = AstNode::SimpleCommand {
            name,
            args,
            tokens: command_tokens,
        };
        
        // Apply assignments by wrapping command if needed
        for assignment in assignments.into_iter().rev() {
            // In actual execution, assignments would modify environment
            // For now, we'll create a compound structure
            command = AstNode::CompoundCommand {
                commands: vec![Box::new(assignment), Box::new(command)],
                tokens: Vec::new(),
            };
        }
        
        // Parse redirections
        command = self.parse_redirections(command)?;
        
        // Check for background operator
        if self.check_token_type(&TokenType::Ampersand) {
            let token = self.current_token.clone().unwrap();
            self.advance();
            
            command = AstNode::Background {
                command: Box::new(command),
                token,
            };
        }
        
        Ok(command)
    }
    
    /// Parse redirections for a command
    fn parse_redirections(&mut self, command: AstNode) -> Result<AstNode, ParseError> {
        let mut current_command = command;
        
        while let Some(token) = self.current_token.clone() {
            let token_type = token.token_type.clone();
            let (redirect_type, fd, target) = match token_type {
                TokenType::Less => (RedirectType::Input, None, self.parse_redirect_target()?),
                TokenType::Great => (RedirectType::Output, None, self.parse_redirect_target()?),
                TokenType::DLess => (RedirectType::HereDoc, None, self.parse_redirect_target()?),
                TokenType::DGreat => (RedirectType::Append, None, self.parse_redirect_target()?),
                TokenType::LessAnd => (RedirectType::DupInput, None, self.parse_redirect_target()?),
                TokenType::GreatAnd => (RedirectType::DupOutput, None, self.parse_redirect_target()?),
                TokenType::LessGreat => (RedirectType::ReadWrite, None, self.parse_redirect_target()?),
                TokenType::DLessDash => (RedirectType::HereDocStrip, None, self.parse_redirect_target()?),
                TokenType::Clobber => (RedirectType::Clobber, None, self.parse_redirect_target()?),
                _ => break,
            };
            
            current_command = AstNode::Redirection {
                command: Box::new(current_command),
                redirect_type,
                target,
                fd,
                token,
            };
        }
        
        Ok(current_command)
    }
    
    /// Parse redirection target (file descriptor or filename)
    fn parse_redirect_target(&mut self) -> Result<String, ParseError> {
        self.advance(); // Skip redirect operator
        
        if let Some(token) = self.current_token.clone() {
            if let TokenType::Word(word) = token.token_type {
                self.advance();
                return Ok(word.clone());
            }
        }
        
        Err(ParseError {
            message: "Expected filename or file descriptor after redirection operator".to_string(),
            token: self.current_token.clone().unwrap_or_else(|| 
                Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
        })
    }
    
    /// Parse if statement
    fn parse_if_statement(&mut self) -> Result<AstNode, ParseError> {
        let mut tokens = Vec::new();
        
        // Parse 'if'
        let if_token = self.current_token.clone().unwrap();
        tokens.push(if_token);
        self.advance();
        
        // Parse condition
        let condition = self.parse_command()?;
        
        // Parse 'then'
        if !self.check_token_type(&TokenType::Then) {
            return Err(ParseError {
                message: "Expected 'then' after if condition".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| 
                    Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
            });
        }
        let then_token = self.current_token.clone().unwrap();
        tokens.push(then_token);
        self.advance();
        
        // Parse then branch
        let mut then_branch = Vec::new();
        while !self.check_token_type(&TokenType::Elif) && 
              !self.check_token_type(&TokenType::Else) && 
              !self.check_token_type(&TokenType::Fi) {
            then_branch.push(Box::new(self.parse_command()?));
        }
        
        // Parse elif branches
        let mut elif_branches = Vec::new();
        while self.check_token_type(&TokenType::Elif) {
            let elif_token = self.current_token.clone().unwrap();
            tokens.push(elif_token);
            self.advance();
            
            let elif_condition = self.parse_command()?;
            
            if !self.check_token_type(&TokenType::Then) {
                return Err(ParseError {
                    message: "Expected 'then' after elif condition".to_string(),
                    token: self.current_token.clone().unwrap_or_else(|| 
                        Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
                });
            }
            let elif_then_token = self.current_token.clone().unwrap();
            tokens.push(elif_then_token);
            self.advance();
            
            let mut elif_body = Vec::new();
            while !self.check_token_type(&TokenType::Elif) && 
                  !self.check_token_type(&TokenType::Else) && 
                  !self.check_token_type(&TokenType::Fi) {
                elif_body.push(Box::new(self.parse_command()?));
            }
            
            elif_branches.push((Box::new(elif_condition), elif_body));
        }
        
        // Parse else branch
        let mut else_branch = None;
        if self.check_token_type(&TokenType::Else) {
            let else_token = self.current_token.clone().unwrap();
            tokens.push(else_token);
            self.advance();
            
            let mut else_body = Vec::new();
            while !self.check_token_type(&TokenType::Fi) {
                else_body.push(Box::new(self.parse_command()?));
            }
            
            else_branch = Some(else_body);
        }
        
        // Parse 'fi'
        if !self.check_token_type(&TokenType::Fi) {
            return Err(ParseError {
                message: "Expected 'fi' to close if statement".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| 
                    Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
            });
        }
        let fi_token = self.current_token.clone().unwrap();
        tokens.push(fi_token);
        self.advance();
        
        Ok(AstNode::IfStatement {
            condition: Box::new(condition),
            then_branch,
            else_branch,
            elif_branches,
            tokens,
        })
    }
    
    /// Parse while loop
    fn parse_while_loop(&mut self) -> Result<AstNode, ParseError> {
        let mut tokens = Vec::new();
        
        // Parse 'while'
        let while_token = self.current_token.clone().unwrap();
        tokens.push(while_token);
        self.advance();
        
        // Parse condition
        let condition = self.parse_command()?;
        
        // Parse 'do'
        if !self.check_token_type(&TokenType::Do) {
            return Err(ParseError {
                message: "Expected 'do' after while condition".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| 
                    Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
            });
        }
        let do_token = self.current_token.clone().unwrap();
        tokens.push(do_token);
        self.advance();
        
        // Parse body
        let mut body = Vec::new();
        while !self.check_token_type(&TokenType::Done) {
            body.push(Box::new(self.parse_command()?));
        }
        
        // Parse 'done'
        if !self.check_token_type(&TokenType::Done) {
            return Err(ParseError {
                message: "Expected 'done' to close while loop".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| 
                    Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
            });
        }
        let done_token = self.current_token.clone().unwrap();
        tokens.push(done_token);
        self.advance();
        
        Ok(AstNode::WhileLoop {
            condition: Box::new(condition),
            body,
            tokens,
        })
    }
    
    /// Parse until loop
    fn parse_until_loop(&mut self) -> Result<AstNode, ParseError> {
        let mut tokens = Vec::new();
        
        // Parse 'until'
        let until_token = self.current_token.clone().unwrap();
        tokens.push(until_token);
        self.advance();
        
        // Parse condition
        let condition = self.parse_command()?;
        
        // Parse 'do'
        if !self.check_token_type(&TokenType::Do) {
            return Err(ParseError {
                message: "Expected 'do' after until condition".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| 
                    Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
            });
        }
        let do_token = self.current_token.clone().unwrap();
        tokens.push(do_token);
        self.advance();
        
        // Parse body
        let mut body = Vec::new();
        while !self.check_token_type(&TokenType::Done) {
            body.push(Box::new(self.parse_command()?));
        }
        
        // Parse 'done'
        if !self.check_token_type(&TokenType::Done) {
            return Err(ParseError {
                message: "Expected 'done' to close until loop".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| 
                    Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
            });
        }
        let done_token = self.current_token.clone().unwrap();
        tokens.push(done_token);
        self.advance();
        
        Ok(AstNode::UntilLoop {
            condition: Box::new(condition),
            body,
            tokens,
        })
    }
    
    /// Parse for loop
    fn parse_for_loop(&mut self) -> Result<AstNode, ParseError> {
        let mut tokens = Vec::new();
        
        // Parse 'for'
        let for_token = self.current_token.clone().unwrap();
        tokens.push(for_token);
        self.advance();
        
        // Parse variable name
        let variable = if let Some(token) = self.current_token.clone() {
            if let TokenType::Word(name) = token.token_type {
                self.advance();
                name.clone()
            } else {
                return Err(ParseError {
                    message: "Expected variable name after 'for'".to_string(),
                    token: token.clone(),
                });
            }
        } else {
            return Err(ParseError {
                message: "Expected variable name after 'for'".to_string(),
                token: Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1),
            });
        };
        
        // Parse 'in'
        if !self.check_token_type(&TokenType::In) {
            return Err(ParseError {
                message: "Expected 'in' after for variable".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| 
                    Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
            });
        }
        let in_token = self.current_token.clone().unwrap();
        tokens.push(in_token);
        self.advance();
        
        // Parse item list
        let mut items = Vec::new();
        while let Some(token) = &self.current_token {
            if let TokenType::Word(item) = &token.token_type {
                items.push(item.clone());
                self.advance();
            } else if token.token_type == TokenType::Semicolon {
                break;
            } else {
                return Err(ParseError {
                    message: "Expected item in for loop list".to_string(),
                    token: token.clone(),
                });
            }
        }
        
        // Parse 'do'
        if !self.check_token_type(&TokenType::Do) {
            return Err(ParseError {
                message: "Expected 'do' after for loop items".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| 
                    Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
            });
        }
        let do_token = self.current_token.clone().unwrap();
        tokens.push(do_token);
        self.advance();
        
        // Parse body
        let mut body = Vec::new();
        while !self.check_token_type(&TokenType::Done) {
            body.push(Box::new(self.parse_command()?));
        }
        
        // Parse 'done'
        if !self.check_token_type(&TokenType::Done) {
            return Err(ParseError {
                message: "Expected 'done' to close for loop".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| 
                    Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
            });
        }
        let done_token = self.current_token.clone().unwrap();
        tokens.push(done_token);
        self.advance();
        
        Ok(AstNode::ForLoop {
            variable,
            items,
            body,
            tokens,
        })
    }
    
    /// Parse compound command: { commands; }
    fn parse_compound_command(&mut self) -> Result<AstNode, ParseError> {
        let mut tokens = Vec::new();
        
        // Parse '{'
        let lbrace_token = self.current_token.clone().unwrap();
        tokens.push(lbrace_token);
        self.advance();
        
        // Parse commands
        let mut commands = Vec::new();
        while !self.check_token_type(&TokenType::RightBrace) {
            commands.push(Box::new(self.parse_command()?));
        }
        
        // Parse '}'
        if !self.check_token_type(&TokenType::RightBrace) {
            return Err(ParseError {
                message: "Expected '}' to close compound command".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| 
                    Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
            });
        }
        let rbrace_token = self.current_token.clone().unwrap();
        tokens.push(rbrace_token);
        self.advance();
        
        Ok(AstNode::CompoundCommand {
            commands,
            tokens,
        })
    }
    
    /// Parse function definition
    fn parse_function_definition(&mut self) -> Result<AstNode, ParseError> {
        let mut tokens = Vec::new();
        
        // Parse 'function' (optional)
        let function_token = self.current_token.clone().unwrap();
        tokens.push(function_token);
        self.advance();
        
        // Parse function name
        let name = if let Some(token) = self.current_token.clone() {
            if let TokenType::Word(name) = token.token_type {
                self.advance();
                name.clone()
            } else {
                return Err(ParseError {
                    message: "Expected function name".to_string(),
                    token: token.clone(),
                });
            }
        } else {
            return Err(ParseError {
                message: "Expected function name".to_string(),
                token: Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1),
            });
        };
        
        // Parse '()' (optional in some shells)
        if self.check_token_type(&TokenType::LeftParen) {
            let lparen_token = self.current_token.clone().unwrap();
            tokens.push(lparen_token);
            self.advance();
            
            if !self.check_token_type(&TokenType::RightParen) {
                return Err(ParseError {
                    message: "Expected ')' after '(' in function definition".to_string(),
                    token: self.current_token.clone().unwrap_or_else(|| 
                        Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
                });
            }
            let rparen_token = self.current_token.clone().unwrap();
            tokens.push(rparen_token);
            self.advance();
        }
        
        // Parse function body (compound command)
        let body = if let Ok(compound) = self.parse_compound_command() {
            compound
        } else {
            return Err(ParseError {
                message: "Expected compound command for function body".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| 
                    Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
            });
        };
        
        // Extract commands from compound command body
        let body_commands = if let AstNode::CompoundCommand { commands, .. } = body {
            commands
        } else {
            Vec::new()
        };
        
        Ok(AstNode::FunctionDefinition {
            name,
            body: body_commands,
            tokens,
        })
    }
    
    /// Parse subshell: ( commands )
    fn parse_subshell(&mut self) -> Result<AstNode, ParseError> {
        let mut tokens = Vec::new();
        
        // Parse '('
        let lparen_token = self.current_token.clone().unwrap();
        tokens.push(lparen_token);
        self.advance();
        
        // Parse commands
        let mut commands = Vec::new();
        while !self.check_token_type(&TokenType::RightParen) {
            commands.push(Box::new(self.parse_command()?));
        }
        
        // Parse ')'
        if !self.check_token_type(&TokenType::RightParen) {
            return Err(ParseError {
                message: "Expected ')' to close subshell".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| 
                    Token::new(TokenType::Error("No token".to_string()), "".to_string(), 1, 1)),
            });
        }
        let rparen_token = self.current_token.clone().unwrap();
        tokens.push(rparen_token);
        self.advance();
        
        Ok(AstNode::Subshell {
            commands,
            tokens,
        })
    }
    
    // Helper methods
    
    fn advance(&mut self) {
        self.current_token = self.tokens.next();
    }
    
    fn peek_token_type(&mut self) -> Option<&TokenType> {
        self.tokens.peek().map(|t| &t.token_type)
    }
    
    fn check_token_type(&mut self, token_type: &TokenType) -> bool {
        if let Some(current) = &self.current_token {
            match (&current.token_type, token_type) {
                (TokenType::Word(_), TokenType::Word(_)) => true,
                (TokenType::AssignmentWord(_), TokenType::AssignmentWord(_)) => true,
                (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
            }
        } else {
            false
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.current_token.is_none()
    }
}