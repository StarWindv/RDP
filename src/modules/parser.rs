//! Parser for shell script AST generation

use std::iter::Peekable;
use std::vec::IntoIter;

use crate::modules::ast::{AstNode, CommandSeparator, ParseError, RedirectType};
use crate::modules::lexer::Lexer;
use crate::modules::tokens::{Token, TokenType};

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
            Ok(ast) => println!(
                "DEBUG PARSER: Parse successful, AST type: {:?}",
                std::mem::discriminant(ast)
            ),
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
        } else if self.check_token_type(&TokenType::Break) {
            return self.parse_break_statement();
        } else if self.check_token_type(&TokenType::Continue) {
            return self.parse_continue_statement();
        } else if self.check_token_type(&TokenType::Return) {
            return self.parse_return_statement();
        }

        // Check for function definition with name() syntax
        if self.is_function_definition_syntax() {
            return self.parse_function_definition_name_syntax();
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
            Ok(AstNode::Pipeline { commands, tokens })
        }
    }

    /// Parse a simple command (with optional redirections and background)
    fn parse_simple_command(&mut self) -> Result<AstNode, ParseError> {
        let mut assignments = Vec::new();
        let mut name = None;
        let mut args = Vec::new();
        let mut command_tokens = Vec::new();
        let mut current_arg = String::new(); // Track current argument being built

        // Check for null command (just a semicolon or newline)
        if let Some(token) = &self.current_token {
            if token.token_type == TokenType::Semicolon
                || token.token_type == TokenType::Newline
            {
                println!(
                    "DEBUG PARSER SIMPLE_CMD: Found separator at start, returning NullCommand: {:?}",
                    token.token_type
                );
                // Consume the separator token (it will be handled by parse_command_list)
                // Actually, we should NOT consume it here - parse_command_list will handle it
                // Just return NullCommand
                return Ok(AstNode::NullCommand);
            }
        }

        // Parse command name and arguments
        while let Some(token) = &self.current_token {
            println!(
                "DEBUG PARSER SIMPLE_CMD: Processing token: {:?}",
                token.token_type
            );
            match &token.token_type {
                TokenType::Word(word) | TokenType::Name(word) | TokenType::QuotedString(word) | TokenType::SingleQuotedString(word) => {
                    println!("DEBUG PARSER SIMPLE_CMD: Word/Name/QuotedString token: '{}'", word);
                    // If we have a current arg being built, add this as a new argument
                    if !current_arg.is_empty() {
                        args.push(current_arg.clone());
                        current_arg.clear();
                    }
                    
                    if name.is_none() {
                        name = Some(word.clone());
                        println!("DEBUG PARSER SIMPLE_CMD: Set command name to '{}'", word);
                    } else {
                        args.push(word.clone());
                        println!(
                            "DEBUG PARSER SIMPLE_CMD: Added argument '{}', args now: {:?}",
                            word, args
                        );
                    }
                    command_tokens.push(token.clone());
                    self.advance();
                }
                TokenType::Number(n) => {
                    // Convert number to string for command arguments
                    let num_str = n.to_string();
                    println!(
                        "DEBUG PARSER SIMPLE_CMD: Number token: {}, converted to '{}'",
                        n, num_str
                    );
                    if name.is_none() {
                        name = Some(num_str.clone());
                        println!(
                            "DEBUG PARSER SIMPLE_CMD: Set command name to '{}' (from number)",
                            num_str
                        );
                    } else {
                        args.push(num_str.clone());
                        println!("DEBUG PARSER SIMPLE_CMD: Added argument '{}' (from number), args now: {:?}", num_str, args);
                    }
                    command_tokens.push(token.clone());
                    self.advance();
                }
                TokenType::AssignmentWord(value) => {
                    println!("DEBUG PARSER SIMPLE_CMD: AssignmentWord token: '{}'", value);
                    if name.is_none() {
                        // This is a variable assignment before command name
                        assignments.push(AstNode::Assignment {
                            name: value.split('=').next().unwrap().to_string(),
                            value: value.splitn(2, '=').nth(1).unwrap_or("").to_string(),
                            token: token.clone(),
                        });
                        println!(
                            "DEBUG PARSER SIMPLE_CMD: Added assignment: {}={}",
                            value.split('=').next().unwrap(),
                            value.splitn(2, '=').nth(1).unwrap_or("")
                        );
                        command_tokens.push(token.clone());
                        self.advance();
                    } else {
                        // This is an argument (e.g., export VAR=value)
                        args.push(value.clone());
                        println!("DEBUG PARSER SIMPLE_CMD: Added argument '{}' (assignment word), args now: {:?}", value, args);
                        command_tokens.push(token.clone());
                        self.advance();
                    }
                }
                TokenType::Dollar => {
                    // $ special variable - treat as word
                    println!("DEBUG PARSER SIMPLE_CMD: Dollar token");
                    let dollar_token = token.clone();
                    command_tokens.push(dollar_token);
                    self.advance();

                    // Check if next token is a variable name
                    if let Some(next_token) = self.current_token.clone() {
                        if let TokenType::Name(var_name) = &next_token.token_type {
                            // This is $VAR syntax
                            println!(
                                "DEBUG PARSER SIMPLE_CMD: Found variable name after $: {}",
                                var_name
                            );
                            command_tokens.push(next_token.clone());
                            self.advance();

                            // Append to current argument
                            let expanded = format!("${}", var_name);
                            if name.is_none() {
                                name = Some(expanded.clone());
                                println!(
                                    "DEBUG PARSER SIMPLE_CMD: Set command name to '{}' (from $VAR)",
                                    expanded
                                );
                            } else {
                                current_arg.push_str(&expanded);
                                println!("DEBUG PARSER SIMPLE_CMD: Appended '{}' to current_arg: '{}'", expanded, current_arg);
                            }
                        } else if let TokenType::Word(var_name) = &next_token.token_type {
                            // This could be $? or other special variable
                            println!("DEBUG PARSER SIMPLE_CMD: Found word after $: {}", var_name);
                            command_tokens.push(next_token.clone());
                            self.advance();

                            // Create expanded argument
                            let expanded = format!("${}", var_name);
                            if name.is_none() {
                                name = Some(expanded.clone());
                                println!("DEBUG PARSER SIMPLE_CMD: Set command name to '{}' (from $word)", expanded);
                            } else {
                                current_arg.push_str(&expanded);
                                println!("DEBUG PARSER SIMPLE_CMD: Appended '{}' to current_arg: '{}'", expanded, current_arg);
                            }
                        } else {
                            // Just a standalone $
                            let expanded = "$".to_string();
                            if name.is_none() {
                                name = Some(expanded.clone());
                                println!("DEBUG PARSER SIMPLE_CMD: Set command name to '{}' (from standalone dollar)", expanded);
                            } else {
                                current_arg.push_str(&expanded);
                                println!("DEBUG PARSER SIMPLE_CMD: Appended '{}' to current_arg: '{}'", expanded, current_arg);
                            }
                        }
                    } else {
                        // Just a $ at end of input
                        let expanded = "$".to_string();
                        if name.is_none() {
                            name = Some(expanded.clone());
                            println!("DEBUG PARSER SIMPLE_CMD: Set command name to '{}' (from dollar at end)", expanded);
                        } else {
                            current_arg.push_str(&expanded);
                            println!("DEBUG PARSER SIMPLE_CMD: Appended '{}' to current_arg: '{}'", expanded, current_arg);
                        }
                    }
                }
                // 分号和换行符是命令分隔符，应该在parse_command_list中处理
                // 当遇到这些token时，我们应该停止解析当前简单命令，让上层处理
                TokenType::Semicolon | TokenType::Newline | TokenType::Ampersand => {
                    // Command separators - these mark the end of the current command
                    // 注意：我们不消费这些token，它们由上层parse_command_list处理
                    println!(
                        "DEBUG PARSER SIMPLE_CMD: Found command separator, stopping: {:?}",
                        token.token_type
                    );
                    break;
                }
                // 管道、逻辑操作符也结束简单命令
                TokenType::Pipe | TokenType::AndIf | TokenType::OrIf => {
                    println!(
                        "DEBUG PARSER SIMPLE_CMD: Found operator, stopping: {:?}",
                        token.token_type
                    );
                    break;
                }
                // 重定向操作符由专门的parse_redirections处理
                TokenType::Less | TokenType::Great | TokenType::DLess | TokenType::DGreat |
                TokenType::LessAnd | TokenType::GreatAnd | TokenType::LessGreat | 
                TokenType::DLessDash | TokenType::Clobber => {
                    // 重定向操作符结束参数解析，由专门的parse_redirections处理
                    println!(
                        "DEBUG PARSER SIMPLE_CMD: Found redirection operator, stopping: {:?}",
                        token.token_type
                    );
                    break;
                }
                // 其他控制结构token也结束简单命令
                TokenType::If | TokenType::Then | TokenType::Else | TokenType::Elif | TokenType::Fi |
                TokenType::While | TokenType::Until | TokenType::Do | TokenType::Done |
                TokenType::For | TokenType::In | TokenType::Case | TokenType::Esac |
                TokenType::Function | TokenType::LeftBrace | TokenType::RightBrace |
                TokenType::LeftParen | TokenType::RightParen |
                TokenType::Break | TokenType::Continue | TokenType::Return => {
                    println!(
                        "DEBUG PARSER SIMPLE_CMD: Found control structure token, stopping: {:?}",
                        token.token_type
                    );
                    break;
                }
                _ => {
                    // Unknown token type, break
                    println!(
                        "DEBUG PARSER SIMPLE_CMD: Unknown token type, breaking: {:?}",
                        token.token_type
                    );
                    break;
                }
            }
        }

        // Push any remaining current_arg
        if !current_arg.is_empty() {
            args.push(current_arg);
            println!("DEBUG PARSER SIMPLE_CMD: Pushed final current_arg");
        }

        // If no command name was found, check if we have assignments only
        if name.is_none() && !assignments.is_empty() {
            // Just assignments, no command - create a compound command with assignments
            let commands: Vec<Box<AstNode>> =
                assignments.into_iter().map(|a| Box::new(a)).collect();
            return Ok(AstNode::CompoundCommand {
                commands,
                tokens: command_tokens,
            });
        }

        // 注意：我们已经处理了NullCommand的情况，所以这里不需要再检查

        let name = name.ok_or_else(|| ParseError {
            message: "Expected command name".to_string(),
            token: self.current_token.clone().unwrap_or_else(|| {
                Token::new(
                    TokenType::Error("No command".to_string()),
                    "".to_string(),
                    1,
                    1,
                )
            }),
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
            // Parse a redirection
                if let Some((redirect_type, fd, target)) = self.parse_redirection()? {
                    current_command = AstNode::Redirection {
                        command: Box::new(current_command),
                        redirect_type,
                        target,
                        fd,
                        token,
                    };
                } else {
                    // Not a redirection, break
                    break;
                }
        }

        Ok(current_command)
    }

    /// Parse a single redirection (could be with file descriptor number)
    fn parse_redirection(&mut self) -> Result<Option<(RedirectType, Option<i32>, String)>, ParseError> {
        // Check if we have a file descriptor number before redirection operator
        let mut fd: Option<i32> = None;
        
        // Look ahead to see if we have a number followed by a redirection operator
        if let Some(token) = self.current_token.clone() {
            if let TokenType::Number(n) = &token.token_type {
                // Save the number as potential file descriptor
                fd = Some(*n);
                
                // Check if next token is a redirection operator
                let saved_tokens = self.tokens.clone();
                let saved_current_token = self.current_token.clone();
                
                // Advance past the number
                self.advance();
                
                if let Some(next_token) = self.current_token.clone() {
                    if next_token.is_redirect_operator() {
                        // This is a numbered redirection (e.g., 2>)
                        let token = next_token;
                        let (redirect_type, target) = self.parse_redirect_operator(&token.token_type)?;
                        
                        // Restore position for actual parsing
                        self.tokens = saved_tokens;
                        self.current_token = saved_current_token;
                        
                        // Now parse for real
                        self.advance(); // Skip number
                        self.advance(); // Skip operator
                        let target_str = self.parse_redirect_target()?;
                        
                        return Ok(Some((redirect_type, fd, target_str)));
                    }
                }
                
                // Not a redirection, restore and continue
                self.tokens = saved_tokens;
                self.current_token = saved_current_token;
            }
        }
        
        // Regular redirection (no file descriptor number)
        if let Some(token) = self.current_token.clone() {
            if token.is_redirect_operator() {
                let redirect_type = match &token.token_type {
                    TokenType::Less => RedirectType::Input,
                    TokenType::Great => RedirectType::Output,
                    TokenType::DLess => RedirectType::HereDoc,
                    TokenType::DGreat => RedirectType::Append,
                    TokenType::LessAnd => RedirectType::DupInput,
                    TokenType::GreatAnd => RedirectType::DupOutput,
                    TokenType::LessGreat => RedirectType::ReadWrite,
                    TokenType::DLessDash => RedirectType::HereDocStrip,
                    TokenType::Clobber => RedirectType::Clobber,
                    _ => return Ok(None), // Not a redirection operator
                };
                
                self.advance();
                let target = self.parse_redirect_target()?;
                return Ok(Some((redirect_type, fd, target)));
            }
        }
        
        Ok(None)
    }
    
    /// Parse redirect operator and return redirect type
    fn parse_redirect_operator(&self, token_type: &TokenType) -> Result<(RedirectType, String), ParseError> {
        match token_type {
            TokenType::Less => Ok((RedirectType::Input, "<".to_string())),
            TokenType::Great => Ok((RedirectType::Output, ">".to_string())),
            TokenType::DLess => Ok((RedirectType::HereDoc, "<<".to_string())),
            TokenType::DGreat => Ok((RedirectType::Append, ">>".to_string())),
            TokenType::LessAnd => Ok((RedirectType::DupInput, "<&".to_string())),
            TokenType::GreatAnd => Ok((RedirectType::DupOutput, ">&".to_string())),
            TokenType::LessGreat => Ok((RedirectType::ReadWrite, "<>".to_string())),
            TokenType::DLessDash => Ok((RedirectType::HereDocStrip, "<<-".to_string())),
            TokenType::Clobber => Ok((RedirectType::Clobber, ">|".to_string())),
            _ => Err(ParseError {
                message: "Not a redirect operator".to_string(),
                token: Token::new(
                    TokenType::Error("Invalid operator".to_string()),
                    "".to_string(),
                    1,
                    1,
                ),
            }),
        }
    }

    /// Parse redirection target (file descriptor, filename, or here-doc delimiter)
    fn parse_redirect_target(&mut self) -> Result<String, ParseError> {
        if let Some(token) = self.current_token.clone() {
            match token.token_type {
                TokenType::Word(word) => {
                    self.advance();
                    return Ok(word.clone());
                }
                TokenType::Number(n) => {
                    self.advance();
                    return Ok(n.to_string());
                }
                TokenType::Name(name) => {
                    self.advance();
                    return Ok(name.clone());
                }
                TokenType::QuotedString(s) => {
                    self.advance();
                    return Ok(s.clone());
                }
                TokenType::SingleQuotedString(s) => {
                    self.advance();
                    return Ok(s.clone());
                }
                TokenType::AssignmentWord(value) => {
                    self.advance();
                    return Ok(value.clone());
                }
                // For here-document, the delimiter is a word
                TokenType::HereDocDelimiter(delim) => {
                    self.advance();
                    return Ok(delim.clone());
                }
                _ => {}
            }
        }

        Err(ParseError {
            message: "Expected filename, file descriptor, or here-document delimiter after redirection operator".to_string(),
            token: self.current_token.clone().unwrap_or_else(|| {
                Token::new(
                    TokenType::Error("No token".to_string()),
                    "".to_string(),
                    1,
                    1,
                )
            }),
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

        // Allow optional semicolon or newline before 'then'
        if self.check_token_type(&TokenType::Semicolon)
            || self.check_token_type(&TokenType::Newline)
        {
            let sep_token = self.current_token.clone().unwrap();
            tokens.push(sep_token);
            self.advance();
        }
        if !self.check_token_type(&TokenType::Then) {
            return Err(ParseError {
                message: "Expected 'then' after if condition".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
            });
        }
        let then_token = self.current_token.clone().unwrap();
        tokens.push(then_token);
        self.advance();

        // Parse then branch
        let mut then_branch = Vec::new();
        while !self.check_token_type(&TokenType::Elif)
            && !self.check_token_type(&TokenType::Else)
            && !self.check_token_type(&TokenType::Fi)
            && !self.is_at_end()
        {
            // Skip semicolons and newlines between commands
            if self.check_token_type(&TokenType::Semicolon)
                || self.check_token_type(&TokenType::Newline)
            {
                self.advance();
                continue;
            }
            then_branch.push(Box::new(self.parse_command()?));
        }

        // Parse elif branches
        let mut elif_branches = Vec::new();
        while self.check_token_type(&TokenType::Elif) {
            let elif_token = self.current_token.clone().unwrap();
            tokens.push(elif_token);
            self.advance();

            let elif_condition = self.parse_command()?;

            // Skip semicolon/newline after condition
            if self.check_token_type(&TokenType::Semicolon)
                || self.check_token_type(&TokenType::Newline)
            {
                self.advance();
            }

            if !self.check_token_type(&TokenType::Then) {
                return Err(ParseError {
                    message: "Expected 'then' after elif condition".to_string(),
                    token: self.current_token.clone().unwrap_or_else(|| {
                        Token::new(
                            TokenType::Error("No token".to_string()),
                            "".to_string(),
                            1,
                            1,
                        )
                    }),
                });
            }
            let elif_then_token = self.current_token.clone().unwrap();
            tokens.push(elif_then_token);
            self.advance();

            let mut elif_body = Vec::new();
            while !self.check_token_type(&TokenType::Elif)
                && !self.check_token_type(&TokenType::Else)
                && !self.check_token_type(&TokenType::Fi)
                && !self.is_at_end()
            {
                // Skip semicolons and newlines between commands
                if self.check_token_type(&TokenType::Semicolon)
                    || self.check_token_type(&TokenType::Newline)
                {
                    self.advance();
                    continue;
                }
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
            while !self.check_token_type(&TokenType::Fi) && !self.is_at_end() {
                // Skip semicolons and newlines between commands
                if self.check_token_type(&TokenType::Semicolon)
                    || self.check_token_type(&TokenType::Newline)
                {
                    self.advance();
                    continue;
                }
                else_body.push(Box::new(self.parse_command()?));
            }

            else_branch = Some(else_body);
        }

        // Parse 'fi'
        if !self.check_token_type(&TokenType::Fi) {
            return Err(ParseError {
                message: "Expected 'fi' to close if statement".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
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

        // Skip semicolon/newline after condition
        if self.check_token_type(&TokenType::Semicolon)
            || self.check_token_type(&TokenType::Newline)
        {
            self.advance();
        }

        // Parse 'do'
        if !self.check_token_type(&TokenType::Do) {
            return Err(ParseError {
                message: "Expected 'do' after while condition".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
            });
        }
        let do_token = self.current_token.clone().unwrap();
        tokens.push(do_token);
        self.advance();

        // Parse body
        let mut body = Vec::new();
        while !self.check_token_type(&TokenType::Done) && !self.is_at_end() {
            // Skip semicolons and newlines between commands
            if self.check_token_type(&TokenType::Semicolon)
                || self.check_token_type(&TokenType::Newline)
            {
                self.advance();
                continue;
            }
            body.push(Box::new(self.parse_command()?));
        }

        // Parse 'done'
        if !self.check_token_type(&TokenType::Done) {
            return Err(ParseError {
                message: "Expected 'done' to close while loop".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
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

        // Skip semicolon/newline after condition
        if self.check_token_type(&TokenType::Semicolon)
            || self.check_token_type(&TokenType::Newline)
        {
            self.advance();
        }

        // Parse 'do'
        if !self.check_token_type(&TokenType::Do) {
            return Err(ParseError {
                message: "Expected 'do' after until condition".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
            });
        }
        let do_token = self.current_token.clone().unwrap();
        tokens.push(do_token);
        self.advance();

        // Parse body
        let mut body = Vec::new();
        while !self.check_token_type(&TokenType::Done) && !self.is_at_end() {
            // Skip semicolons and newlines between commands
            if self.check_token_type(&TokenType::Semicolon)
                || self.check_token_type(&TokenType::Newline)
            {
                self.advance();
                continue;
            }
            body.push(Box::new(self.parse_command()?));
        }

        // Parse 'done'
        if !self.check_token_type(&TokenType::Done) {
            return Err(ParseError {
                message: "Expected 'done' to close until loop".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
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
            if let TokenType::Name(name) = token.token_type {
                self.advance();
                name.clone()
            } else if let TokenType::Word(name) = token.token_type {
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
                token: Token::new(
                    TokenType::Error("No token".to_string()),
                    "".to_string(),
                    1,
                    1,
                ),
            });
        };

        // Parse 'in'
        if !self.check_token_type(&TokenType::In) {
            return Err(ParseError {
                message: "Expected 'in' after for variable".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
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
            } else if let TokenType::Name(item) = &token.token_type {
                items.push(item.clone());
                self.advance();
            } else if token.token_type == TokenType::Semicolon
                || token.token_type == TokenType::Newline
            {
                break;
            } else {
                return Err(ParseError {
                    message: "Expected item in for loop list".to_string(),
                    token: token.clone(),
                });
            }
        }

        // Skip semicolon/newline after items
        if self.check_token_type(&TokenType::Semicolon)
            || self.check_token_type(&TokenType::Newline)
        {
            self.advance();
        }

        // Parse 'do'
        if !self.check_token_type(&TokenType::Do) {
            return Err(ParseError {
                message: "Expected 'do' after for loop items".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
            });
        }
        let do_token = self.current_token.clone().unwrap();
        tokens.push(do_token);
        self.advance();

        // Parse body
        let mut body = Vec::new();
        while !self.check_token_type(&TokenType::Done) && !self.is_at_end() {
            // Skip semicolons and newlines between commands
            if self.check_token_type(&TokenType::Semicolon)
                || self.check_token_type(&TokenType::Newline)
            {
                self.advance();
                continue;
            }
            body.push(Box::new(self.parse_command()?));
        }

        // Parse 'done'
        if !self.check_token_type(&TokenType::Done) {
            return Err(ParseError {
                message: "Expected 'done' to close for loop".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
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
        while !self.check_token_type(&TokenType::RightBrace) && !self.is_at_end() {
            // Skip semicolons and newlines between commands
            if self.check_token_type(&TokenType::Semicolon)
                || self.check_token_type(&TokenType::Newline)
            {
                self.advance();
                continue;
            }
            commands.push(Box::new(self.parse_command()?));
        }

        // Parse '}'
        if !self.check_token_type(&TokenType::RightBrace) {
            return Err(ParseError {
                message: "Expected '}' to close compound command".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
            });
        }
        let rbrace_token = self.current_token.clone().unwrap();
        tokens.push(rbrace_token);
        self.advance();

        Ok(AstNode::CompoundCommand { commands, tokens })
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
                token: Token::new(
                    TokenType::Error("No token".to_string()),
                    "".to_string(),
                    1,
                    1,
                ),
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
                    token: self.current_token.clone().unwrap_or_else(|| {
                        Token::new(
                            TokenType::Error("No token".to_string()),
                            "".to_string(),
                            1,
                            1,
                        )
                    }),
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
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
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
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
            });
        }
        let rparen_token = self.current_token.clone().unwrap();
        tokens.push(rparen_token);
        self.advance();

        Ok(AstNode::Subshell { commands, tokens })
    }

    /// Check if the current tokens represent a function definition with name() syntax
    fn is_function_definition_syntax(&mut self) -> bool {
        // Save current position
        let saved_tokens = self.tokens.clone();
        let saved_current_token = self.current_token.clone();

        // Check for pattern: Word/Name + LeftParen + RightParen + LeftBrace
        // First token should be a word/name
        let first_is_word = if let Some(token) = &self.current_token {
            matches!(&token.token_type, TokenType::Word(_) | TokenType::Name(_))
        } else {
            false
        };

        if !first_is_word {
            // Restore position
            self.tokens = saved_tokens;
            self.current_token = saved_current_token;
            return false;
        }

        // Advance past the name
        self.advance();

        // Check for LeftParen
        let has_lparen = self.check_token_type(&TokenType::LeftParen);
        if !has_lparen {
            // Restore position
            self.tokens = saved_tokens;
            self.current_token = saved_current_token;
            return false;
        }
        self.advance();

        // Check for RightParen
        let has_rparen = self.check_token_type(&TokenType::RightParen);
        if !has_rparen {
            // Restore position
            self.tokens = saved_tokens;
            self.current_token = saved_current_token;
            return false;
        }
        self.advance();

        // Check for LeftBrace
        let has_lbrace = self.check_token_type(&TokenType::LeftBrace);

        // Restore position
        self.tokens = saved_tokens;
        self.current_token = saved_current_token;

        has_lbrace
    }

    /// Parse function definition with name() syntax
    fn parse_function_definition_name_syntax(&mut self) -> Result<AstNode, ParseError> {
        let mut tokens = Vec::new();

        // Parse function name
        let name_token = self.current_token.clone().unwrap();
        let name = if let TokenType::Word(name) | TokenType::Name(name) = &name_token.token_type {
            tokens.push(name_token.clone());
            self.advance();
            name.clone()
        } else {
            return Err(ParseError {
                message: "Expected function name".to_string(),
                token: name_token.clone(),
            });
        };

        // Parse '('
        if !self.check_token_type(&TokenType::LeftParen) {
            return Err(ParseError {
                message: "Expected '(' after function name".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
            });
        }
        let lparen_token = self.current_token.clone().unwrap();
        tokens.push(lparen_token);
        self.advance();

        // Parse ')'
        if !self.check_token_type(&TokenType::RightParen) {
            return Err(ParseError {
                message: "Expected ')' after '(' in function definition".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
            });
        }
        let rparen_token = self.current_token.clone().unwrap();
        tokens.push(rparen_token);
        self.advance();

        // Parse function body (compound command)
        let body = if let Ok(compound) = self.parse_compound_command() {
            compound
        } else {
            return Err(ParseError {
                message: "Expected compound command for function body".to_string(),
                token: self.current_token.clone().unwrap_or_else(|| {
                    Token::new(
                        TokenType::Error("No token".to_string()),
                        "".to_string(),
                        1,
                        1,
                    )
                }),
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

    /// Parse break statement: break [n]
    fn parse_break_statement(&mut self) -> Result<AstNode, ParseError> {
        let token = self.current_token.clone().unwrap();
        self.advance();

        // Parse optional level argument (break [n])
        let level = if !self.is_at_end() &&
            !self.check_token_type(&TokenType::Semicolon) &&
            !self.check_token_type(&TokenType::Newline) &&
            !self.check_token_type(&TokenType::Pipe) &&
            !self.check_token_type(&TokenType::AndIf) &&
            !self.check_token_type(&TokenType::OrIf) &&
            !self.check_token_type(&TokenType::Ampersand) {
            if let Some(tok) = &self.current_token {
                if let TokenType::Number(n) = &tok.token_type {
                    let level = *n as u32;
                    self.advance();
                    Some(level)
                } else if let TokenType::Word(s) = &tok.token_type {
                    if let Ok(n) = s.parse::<u32>() {
                        self.advance();
                        Some(n)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(AstNode::BreakStatement { level, token })
    }

    /// Parse continue statement: continue [n]
    fn parse_continue_statement(&mut self) -> Result<AstNode, ParseError> {
        let token = self.current_token.clone().unwrap();
        self.advance();

        // Parse optional level argument (continue [n])
        let level = if !self.is_at_end() &&
            !self.check_token_type(&TokenType::Semicolon) &&
            !self.check_token_type(&TokenType::Newline) &&
            !self.check_token_type(&TokenType::Pipe) &&
            !self.check_token_type(&TokenType::AndIf) &&
            !self.check_token_type(&TokenType::OrIf) &&
            !self.check_token_type(&TokenType::Ampersand) {
            if let Some(tok) = &self.current_token {
                if let TokenType::Number(n) = &tok.token_type {
                    let level = *n as u32;
                    self.advance();
                    Some(level)
                } else if let TokenType::Word(s) = &tok.token_type {
                    if let Ok(n) = s.parse::<u32>() {
                        self.advance();
                        Some(n)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(AstNode::ContinueStatement { level, token })
    }

    /// Parse return statement: return [status]
    fn parse_return_statement(&mut self) -> Result<AstNode, ParseError> {
        let token = self.current_token.clone().unwrap();
        self.advance();

        // Parse optional exit code argument (return [status])
        let exit_code = if !self.is_at_end() &&
            !self.check_token_type(&TokenType::Semicolon) &&
            !self.check_token_type(&TokenType::Newline) &&
            !self.check_token_type(&TokenType::Pipe) &&
            !self.check_token_type(&TokenType::AndIf) &&
            !self.check_token_type(&TokenType::OrIf) &&
            !self.check_token_type(&TokenType::Ampersand) {
            if let Some(tok) = &self.current_token {
                match &tok.token_type {
                    TokenType::Number(n) => {
                        let code = n.to_string();
                        self.advance();
                        Some(code)
                    }
                    TokenType::Word(s) => {
                        let code = s.clone();
                        self.advance();
                        Some(code)
                    }
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(AstNode::ReturnStatement { exit_code, token })
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
