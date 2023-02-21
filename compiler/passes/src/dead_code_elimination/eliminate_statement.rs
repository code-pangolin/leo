// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

use crate::{DeadCodeEliminator, FunctionInliner};

use leo_ast::{
    AssertStatement, AssertVariant, AssignStatement, Block, ConditionalStatement, ConsoleStatement, DecrementStatement,
    DefinitionStatement, Expression, ExpressionReconstructor, ExpressionStatement, IncrementStatement,
    IterationStatement, ReturnStatement, Statement, StatementReconstructor,
};

impl StatementReconstructor for DeadCodeEliminator {
    fn reconstruct_assert(&mut self, input: AssertStatement) -> (Statement, Self::AdditionalOutput) {
        // Set the `is_necessary` flag.
        self.is_necessary = true;

        // Visit the statement.
        let statement = Statement::Assert(AssertStatement {
            variant: match input.variant {
                AssertVariant::Assert(expr) => AssertVariant::Assert(self.reconstruct_expression(expr).0),
                AssertVariant::AssertEq(left, right) => AssertVariant::AssertEq(
                    self.reconstruct_expression(left).0,
                    self.reconstruct_expression(right).0,
                ),
                AssertVariant::AssertNeq(left, right) => AssertVariant::AssertNeq(
                    self.reconstruct_expression(left).0,
                    self.reconstruct_expression(right).0,
                ),
            },
            span: input.span,
        });

        // Unset the `is_necessary` flag.
        self.is_necessary = false;

        (statement, Default::default())
    }

    /// Reconstruct an assignment statement by eliminating any dead code.
    fn reconstruct_assign(&mut self, input: AssignStatement) -> (Statement, Self::AdditionalOutput) {
        // Check the lhs of the assignment to see any of variables are used.
        let lhs_is_used = match &input.place {
            Expression::Identifier(identifier) => self.used_variables.contains(&identifier.name),
            Expression::Tuple(tuple_expression) => tuple_expression
                .elements
                .iter()
                .map(|element| match element {
                    Expression::Identifier(identifier) => identifier.name,
                    _ => unreachable!(
                        "The previous compiler passes guarantee the tuple elements on the lhs are identifiers."
                    ),
                })
                .any(|symbol| self.used_variables.contains(&symbol)),
            _ => unreachable!(
                "The previous compiler passes guarantee that `place` is either an identifier or tuple of identifiers."
            ),
        };

        println!("self.used_variables: {:?}", self.used_variables);
        println!("Statement: {}, lhs_is_used: {:?}", input, lhs_is_used);

        match lhs_is_used {
            // If the lhs is used, then we return the original statement.
            true => {
                // Set the `is_necessary` flag.
                self.is_necessary = true;

                // Visit the statement.
                let statement = Statement::Assign(Box::new(AssignStatement {
                    place: input.place,
                    value: self.reconstruct_expression(input.value).0,
                    span: input.span,
                }));

                // Unset the `is_necessary` flag.
                self.is_necessary = false;

                (statement, Default::default())
            },
            // Otherwise, we can eliminate it.
            false => (Statement::dummy(Default::default()), Default::default()),
        }
    }

    /// Reconstructs the statements inside a basic block, eliminating any dead code.
    fn reconstruct_block(&mut self, block: Block) -> (Block, Self::AdditionalOutput) {
        // Reconstruct each of the statements in reverse.
        let mut statements: Vec<Statement> = block
            .statements
            .into_iter()
            .rev()
            .map(|statement| {
                println!("Reconstructing statement: {}", statement);
                self.reconstruct_statement(statement).0
            }).collect();

        // Reverse the direction of `statements`.
        statements.reverse();

        (
            Block {
                statements,
                span: block.span,
            },
            Default::default(),
        )
    }

    /// Flattening removes conditional statements from the program.
    fn reconstruct_conditional(&mut self, _: ConditionalStatement) -> (Statement, Self::AdditionalOutput) {
        unreachable!("`ConditionalStatement`s should not be in the AST at this phase of compilation.")
    }

    /// Parsing guarantees that console statements are not present in the program.
    fn reconstruct_console(&mut self, _: ConsoleStatement) -> (Statement, Self::AdditionalOutput) {
        unreachable!("`ConsoleStatement`s should not be in the AST at this phase of compilation.")
    }

    fn reconstruct_decrement(&mut self, input: DecrementStatement) -> (Statement, Self::AdditionalOutput) {
        // Set the `is_necessary` flag.
        self.is_necessary = true;

        // Visit the statement.
        let statement = Statement::Decrement(DecrementStatement {
            mapping: input.mapping,
            index: self.reconstruct_expression(input.index).0,
            amount: self.reconstruct_expression(input.amount).0,
            span: input.span,
        });

        // Unset the `is_necessary` flag.
        self.is_necessary = false;

        (statement, Default::default())
    }

    /// Static single assignment replaces definition statements with assignment statements.
    fn reconstruct_definition(&mut self, _: DefinitionStatement) -> (Statement, Self::AdditionalOutput) {
        unreachable!("`DefinitionStatement`s should not exist in the AST at this phase of compilation.")
    }

    /// Reconstructs expression statements by eliminating any dead code.
    fn reconstruct_expression_statement(&mut self, input: ExpressionStatement) -> (Statement, Self::AdditionalOutput) {
        match input.expression {
            Expression::Call(expression) => {
                // Set the `is_necessary` flag.
                self.is_necessary = true;

                // Visit the expression.
                let statement = Statement::Expression(ExpressionStatement {
                    expression: self.reconstruct_call(expression).0,
                    span: input.span,
                });

                // Unset the `is_necessary` flag.
                self.is_necessary = false;

                (statement, Default::default())
            }
            _ => unreachable!("Type checking guarantees that expression statements are always function calls."),
        }
    }

    fn reconstruct_increment(&mut self, input: IncrementStatement) -> (Statement, Self::AdditionalOutput) {
        // Set the `is_necessary` flag.
        self.is_necessary = true;

        // Visit the statement.
        let statement = Statement::Increment(IncrementStatement {
            mapping: input.mapping,
            index: self.reconstruct_expression(input.index).0,
            amount: self.reconstruct_expression(input.amount).0,
            span: input.span,
        });

        // Unset the `is_necessary` flag.
        self.is_necessary = false;

        (statement, Default::default())
    }

    /// Loop unrolling unrolls and removes iteration statements from the program.
    fn reconstruct_iteration(&mut self, _: IterationStatement) -> (Statement, Self::AdditionalOutput) {
        unreachable!("`IterationStatement`s should not be in the AST at this phase of compilation.");
    }

    fn reconstruct_return(&mut self, input: ReturnStatement) -> (Statement, Self::AdditionalOutput) {
        // Set the `is_necessary` flag.
        self.is_necessary = true;

        // Visit the statement.
        let statement = Statement::Return(ReturnStatement {
            expression: self.reconstruct_expression(input.expression).0,
            finalize_arguments: input.finalize_arguments.map(|arguments| {
                arguments
                    .into_iter()
                    .map(|argument| self.reconstruct_expression(argument).0)
                    .collect()
            }),
            span: input.span,
        });

        // Unset the `is_necessary` flag.
        self.is_necessary = false;

        (statement, Default::default())
    }
}
