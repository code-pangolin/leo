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

//! The Dead Code Elimination pass traverses the AST and eliminates unused code.
//! See https://en.wikipedia.org/wiki/Dead-code_elimination for more information.
//!
//! Consider the following flattened Leo code.
//! ```leo
//! function main(flag: u8, value: u8) -> u8 {
//!     $var$0 = flag == 0u8;
//!     $var$4$5 = value * value;
//!     $var$1 = $var$4$5;
//!     value$2 = $var$1;
//!     value$3 = $var$0 ? value$2 : value;
//!     value$6 = $var$1 * $var$1;
//!     return value$3;
//! }
//! ```
//!
//! The dead code elimination pass produces the following code.
//! ```leo
//! function main(flag: u8, value: u8) -> u8 {
//!     $var$0 = flag == 0u8;
//!     $var$4$5 = value * value;
//!     $var$1 = $var$4$5;
//!     value$2 = $var$1;
//!     value$3 = $var$0 ? value$2 : value;
//!     return value$3;
//! }
//! ```
//!

mod eliminate_expression;

mod eliminate_statement;

mod eliminate_program;

pub mod dead_code_eliminator;
pub use dead_code_eliminator::*;

use crate::Pass;

use leo_ast::{Ast, ProgramReconstructor};
use leo_errors::Result;

impl<'a> Pass for FunctionInliner<'a> {
    type Input = Ast;
    type Output = Result<Ast>;

    fn do_pass(ast: Self::Input) -> Self::Output {
        let mut reconstructor = DeadCodeEliminator::new();
        let program = reconstructor.reconstruct_program(ast.into_repr());

        Ok(Ast::new(program))
    }
}
