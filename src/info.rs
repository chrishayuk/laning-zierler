//! What each word, character and function of the language means, in the
//! words of E-364 where it has words for it. The language server shows these
//! on hover and in completion; nothing else depends on them.
//!
//! Each entry is a heading, a paragraph and an example.

/// A heading, a paragraph and an example.
pub type Help = (&'static str, &'static str, &'static str);

/// The instructions: the upper-case words of the tape.
pub const WORDS: &[&str] = &["PRINT", "STOP", "SP", "CP", "SR", "CR", "ASSIGN", "D", "F", "N"];

pub fn word_help(word: &str) -> Option<Help> {
    Some(match word {
        "PRINT" => (
            "types the current values of letters",
            "\"The current value of any number of letter variables may be printed out by inserting the instruction PRINT followed by the desired letters, followed by a period. No computation may be performed during a print instruction (e.g., PRINT 3a. is not admissible) and commas may, but need not, be inserted between letters.\" (p. 6) Each value is typed as `x = +.50000000+01`: eight digits and a power of ten. Six go on a line; each PRINT begins and ends with a carriage return, so two PRINTs are separated by a line space. A PRINT may be numbered; numbered 100, it is carried out after an alarm.",
            "PRINT x, y|3, b|i.\nPRINT xy.",
        ),
        "STOP" => (
            "the end of the program",
            "\"The instruction STOP must be placed at the end of every program. Its effect is to stop the computer. This instruction may be given an equation number so that an SP or CP may be used to stop the computation.\" (p. 9) It \"may appear only at the end of a program\" (rule 8). It needs no comma.",
            "3   STOP",
        ),
        "SP" => (
            "go on from another equation",
            "\"The effect of the instruction SP followed by an integer from 1 to 100 and a comma is to 'transfer control' to the equation with the specified number and to proceed on from that equation.\" (p. 8) The number may be a letter: `SP u,` goes to the equation whose number is the value of u, rounded (p. 15). An SP may not itself be numbered.",
            "SP 3,\nSP u,",
        ),
        "CP" => (
            "go on from another equation if the last result was negative",
            "\"The effect of the instruction CP 5, is the same as that of SP 5, if the last computed quantity is negative while CP 5, is ignored if the last computed quantity is positive.\" (p. 8) A zero is the trap: \"the CP may or may not behave like an SP since 0 may appear as +0 or -0.\" The manual's loops subtract a half (`e = n - 9.5,`) so that the quantity is never zero. Here a zero that arithmetic produces is -0, and CP jumps on it.",
            "    e = n - 9.5,\n    CP 1,",
        ),
        "SR" => (
            "do one equation, then come back",
            "\"The instruction SR 5, causes equation 5 to be executed next and then, after equation 5, the equation or instruction following the SR 5,.\" (p. 9) One equation only: SR is not a call of a block. In January 1954 \"An SR or CR instruction may not refer to an equation that makes use of a subroutine\" (p. 21; `lz --temporary-rules`).",
            "SR 5,",
        ),
        "CR" => (
            "do one equation and come back, if the last result was negative",
            "\"The instruction CR 5, bears the same relation to SR 5, that CP 5, does to SP 5,.\" (p. 9)",
            "CR 5,",
        ),
        "ASSIGN" => (
            "reserves variables on the drum",
            "\"The instruction ASSIGN followed by a lower case letter followed by a vertical slash followed by an upper case integer between 1 and 1023 inclusive followed by a comma causes space on the drum to be selected for variables. For example, ASSIGN a|1000, automatically reserves space for variables a|1 through a|1000 inclusive to the drum and takes up in the storage table for variables in MS the space of only one ordinary superscript variable.\" (p. 15) Arithmetic with a drum variable takes \"about 30 milliseconds longer per variable\" (p. 16).",
            "ASSIGN a|1000,",
        ),
        "D" => (
            "d/dt: a differential equation",
            "\"Provision has been made for the solution of ordinary differential equations by the method of Gill - a variation of the 4th order Runge-Kutta technique.\" (p. 17) \"1) the letter t must be used for the independent variable. 2) the letter h must be used for the increment in t. 3) the upper case letter D is used to denote d/dt.\" (p. 18) Each step of length h goes through the equations from the first D equation to the last four times, and t is increased by h after the last. \"It is essential that all relevant auxiliary computation take place between the first and last D equations.\" (p. 19)",
            "2   D y|1 = y|2 + 1,\n    D y|2 = - y|1,",
        ),
        "F" => (
            "a function subroutine",
            "\"The sine of x may be obtained by writing simply F²(x) and such an expression may be treated arithmetically in the same way as a variable in an equation.\" (p. 12) The number of the function is written as an exponent: F¹ to F²³. \"Functions of functions are permitted.\"",
            "x = 2(F^1(y) F^2(z) + F^11(z)),",
        ),
        "N" => (
            "values for a whole table, on the drum",
            "`g|N = 2, 4, 6, 0,` gives g|1 to g|4 their values and puts them on the drum. \"Any lower case letter may be used in this way but only the upper case letter N may follow the vertical slash. Each value, including the last, must be followed by a comma.\" (p. 16) An increment in parentheses fills in a run: `a|N = 1.0 (.5) 2. (0.25) 2.5 (1) 4.5,` is 1, 1.5, 2, 2.25, 2.5, 3.5, 4.5. In January 1954 \"The x|N is not yet in operation\" (p. 21), and in the first quarter of 1954 every routine had been tested \"with the exception of the x|N routine\" (Summary Report 37).",
            "g|N = 2, 4, 6, 0,\nk|N = 1 (1) 10,",
        ),
        _ => return None,
    })
}

/// F¹ to F²³ (section 11): the name, and what it computes.
pub const FUNCTIONS: [(&str, &str); 23] = crate::num::FUNCTIONS;

/// What F^k is, for hover and completion.
pub fn function_help(k: u32) -> Option<(String, String)> {
    if !(1..=23).contains(&k) {
        return None;
    }
    let (_, what) = FUNCTIONS[k as usize - 1];
    let sup = crate::lex::superscript(k as i64);
    let note = match k {
        1 => " An argument below zero is an alarm.",
        5 | 6 => " An argument beyond 1 is an alarm.",
        8..=10 | 18..=20 => " Where the function is infinite, the alarm is a division by zero.",
        12 => " +1 or -1; zero for zero.",
        21..=23 => " An argument that is not positive is an alarm.",
        _ => "",
    };
    Some((
        format!("F{sup}: {what}"),
        format!("Function subroutine {k} of the 23 in E-364 (p. 12).{note} Written `F{sup}(x)` or `F^{k}(x)`."),
    ))
}

/// The characters of the tape that are not letters or digits.
pub fn symbol_help(symbol: &str) -> Option<Help> {
    Some(match symbol {
        "/" => (
            "division",
            "\"A plus or minus sign has the effect of separating any mathematical expression into two terms.\" Within a term, a slash puts the factors after it on the other side of the fraction, until the next slash: `cd/a(b + c) + d` is cd over a(b+c), plus d; \"a/b/c is properly interpreted as the product ac divided by b\" (pp. 4-5). Not a/(bc), as FORTRAN and most later languages would have it.",
            "x = a + 2b/(c + d),\nx = a/b/c,           # ac/b",
        ),
        "·" | ":" | "*" => (
            "times",
            "\"The lower dot of the colon has been filed off on at least one of the Flexowriter units available at DCL. This character, or the colon itself, may be used to denote multiplication\" (p. 10). Mostly it is not needed: letters, numbers and parentheses written side by side are multiplied, `2ay`. Between two numbers it is: `2·3·4`. A space between digits is a multiplication too, so `2 3` is 6. `*` is accepted here for `·`.",
            "x = x^4/2·3·4,",
        ),
        "+" | "-" | "−" => (
            "plus and minus",
            "Plus and minus \"with their usual significance\"; each separates the expression into terms (p. 4). A minus may begin an expression: `y = -6.3a,`, `w = - w,`. There is an upper-case minus for exponents (`a⁻³`) but no upper-case plus.",
            "w = - w,\nn = n + 2,",
        ),
        "=" => (
            "an equation",
            "\"The computer regards the right hand side of an equation as a set of computational operations to be performed. The result ... is always a number and after the computation is finished this number is stored in the registers assigned to the letter that appears on the left hand side.\" So `n = n + 2,` is meaningful (p. 2).",
            "n = n + 2,",
        ),
        "," => (
            "the end of an equation",
            "\"Every equation must end with a comma\" (rule 1), and so must SP, CP, SR, CR and ASSIGN. A line break means nothing: the tape is one stream of characters.",
            "a = 5, y = -6.3a,",
        ),
        "." => (
            "a decimal point, or the end of a PRINT",
            "\"Every PRINT instruction must end with a period\" (rule 4). In a number it is the decimal point: \"0.012, 012, .012, 48, 321 etc.\" are all read correctly (p. 5).",
            "PRINT x, z.",
        ),
        "|" => (
            "a superscript variable",
            "\"Any lower case letter followed by a vertical slash followed by an upper case integer between 0 and 1023 inclusive may be used as a variable in precisely the same way as the lower case letters alone\" (p. 13). After the slash may also come a letter, whose value is the index (`x|n`), or N (`g|N = ...`). The Flexowriter had no subscripts; the index was typed in upper case, `x|⁵`. Here `x|5` is the same.",
            "x|5 = 2n,\ny = d f|n + (1 - d) f|m,",
        ),
        "(" | ")" => (
            "parentheses",
            "\"Only standard parentheses are available typographically. No braces or square brackets may be used. No more than four parentheses may be open in a single equation at any one time\" (p. 3). The argument of a function is in parentheses too.",
            "x = a(b + c(d + e(f + g(h + i)))),",
        ),
        "^" | "⁻" | "⁰" | "¹" | "²" | "³" | "⁴" | "⁵" | "⁶" | "⁷" | "⁸" | "⁹" => (
            "an exponent",
            "\"Upper case numbers on the Flexowriter appear as exponents; there is an upper case minus sign but no upper case plus sign. The computer will handle positive (unsigned) and negative integral exponents.\" (p. 5) The exponent applies to the factor just before it: a number, a letter, a parenthesis or a function. Here `x^2` and `x^-3` stand for `x²` and `x⁻³`.",
            "b = (a - 2)⁻²,\nd = a(b + c)^2/(c - a)a^-3,",
        ),
        "#" => (
            "a note",
            "Everything from `#` to the end of the line is a note for the reader. Notes are not on the tape: E-364 describes no way of writing a comment, and the translator here never sees them.",
            "x = 0,   # the first value",
        ),
        _ => return None,
    })
}

/// The letters with a meaning of their own when there are D equations.
pub fn letter_role(c: char, has_differential_equations: bool) -> Option<&'static str> {
    if !has_differential_equations {
        return None;
    }
    match c {
        't' => Some("the independent variable of the differential equations: \"the letter t must be used for the independent variable\", and \"t is automatically increased by the value h upon completion of the last equation that starts with a D\" (pp. 18-19)"),
        'h' => Some("the increment in t: \"the letter h must be used for the increment in t\" (p. 18)"),
        _ => None,
    }
}

/// The rule for a letter used as an index or an equation number (p. 13).
pub const INDEX_RULE: &str = "\"If, in the expression x|n, n has a non-integral value greater than 0.5, the computer rounds n off to the nearest integer. Negative values of n are not permitted in this connection, nor are values between zero and one half inclusive, with the exception that the value zero is permissible if assigned by the specific equation n = 0,\" (p. 13). Here the rule is approximated through the sign of zero: a zero left by a subtraction (n = n - 1 from 1) is -0 and is an alarm, but a computed zero that comes out +0 (n = 0·5) is let through, though the rule forbids it.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_word_and_function_has_help() {
        for w in WORDS {
            assert!(word_help(w).is_some(), "{w}");
        }
        for k in 1..=23 {
            assert!(function_help(k).is_some());
        }
        assert!(function_help(24).is_none());
        assert!(function_help(2).unwrap().0.contains("sine"));
        for s in ["/", "·", ":", "*", "+", "-", "=", ",", ".", "|", "(", ")", "^", "²", "#"] {
            assert!(symbol_help(s).is_some(), "{s}");
        }
    }
}
