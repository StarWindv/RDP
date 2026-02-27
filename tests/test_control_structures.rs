/// Tests for control structures: if, for, while, case, break, continue

#[cfg(test)]
mod control_structure_tests {
    use rs_dash_pro::modules::lexer::Lexer;
    use rs_dash_pro::modules::parser::Parser;
    use rs_dash_pro::modules::ssa_ir_generator::SsaIrGenerator;
    use rs_dash_pro::modules::ssa_executor::SsaExecutor;

    fn execute(input: &str) -> i32 {
        // Lexical analysis
        let lexer = Lexer::new(input);
        let _tokens: Vec<_> = lexer.collect();
        
        // Parsing
        let mut parser = Parser::new(input);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                return 1;
            }
        };
        
        // SSA IR generation
        let mut generator = SsaIrGenerator::new();
        let ssa_func = generator.generate(ast);
        
        // Execution
        let mut executor = SsaExecutor::new();
        executor.execute_function(&ssa_func)
    }

    #[test]
    fn test_if_then_success() {
        // if true; then exit 0; fi
        let input = "if true; then exit 0; fi";
        let status = execute(input);
        assert_eq!(status, 0, "if true should succeed");
    }

    #[test]
    fn test_if_then_failure() {
        // if false; then exit 0; fi
        let input = "if false; then exit 0; fi";
        let status = execute(input);
        // if condition fails and no else, should return 0
        assert_eq!(status, 0, "if false with no else should return 0");
    }

    #[test]
    fn test_if_then_else() {
        // if false; then exit 1; else exit 0; fi
        let input = "if false; then exit 1; else exit 0; fi";
        let status = execute(input);
        assert_eq!(status, 0, "if-else should execute else branch");
    }

    #[test]
    fn test_if_elif_else() {
        // if false; then exit 1; elif true; then exit 0; else exit 2; fi
        let input = "if false; then exit 1; elif true; then exit 0; else exit 2; fi";
        let status = execute(input);
        assert_eq!(status, 0, "if-elif-else should execute elif when condition true");
    }

    #[test]
    fn test_simple_for_loop() {
        // for i in 1 2 3; do echo $i; done
        // Should execute 3 times
        let input = "for i in 1 2 3; do echo $i; done";
        let status = execute(input);
        // For now, just check it doesn't crash
        assert_eq!(status, 0, "simple for loop should succeed");
    }

    #[test]
    fn test_while_loop() {
        // Simple while loop test
        // while false; do echo never; done
        let input = "while false; do echo never; done";
        let status = execute(input);
        assert_eq!(status, 0, "while false should exit with 0");
    }

    #[test]
    fn test_case_statement() {
        // case word in pattern) commands;; esac
        let input = "case hello in hello) exit 0;; *) exit 1;; esac";
        let status = execute(input);
        assert_eq!(status, 0, "case statement should match pattern");
    }
}
