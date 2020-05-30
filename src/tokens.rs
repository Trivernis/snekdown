#![allow(unused)]

pub(crate) const BACKSLASH: char = '\\';
pub(crate) const LB: char = '\n';
pub(crate) const ASTERISK: char = '*';
pub(crate) const UNDERSCR: char = '_';
pub(crate) const TILDE: char = '~';
pub(crate) const PIPE: char = '|';
pub(crate) const BACKTICK: char = '`';
pub(crate) const R_BRACKET: char = '[';
pub(crate) const L_BRACKET: char = ']';
pub(crate) const R_PARENTH: char = '(';
pub(crate) const L_PARENTH: char = ')';
pub(crate) const MINUS: char = '-';
pub(crate) const PLUS: char = '+';
pub(crate) const HASH: char = '#';
pub(crate) const O: char = 'o';
pub(crate) const X: char = 'x';

// aliases

pub(crate) const SPECIAL_ESCAPE: char = BACKSLASH;

// groups

pub(crate) const BLOCK_SPECIAL_CHARS: [char; 4] = [HASH, MINUS, BACKTICK, PIPE];
pub(crate) const INLINE_SPECIAL_CHARS: [char; 6] = [LB, ASTERISK, UNDERSCR, TILDE, PIPE, BACKTICK];

pub(crate) const LIST_SPECIAL_CHARS: [char; 4] = [MINUS, PLUS, ASTERISK, O];

// sequences

pub(crate) const SQ_CODE_BLOCK: [char; 3] = [BACKTICK, BACKTICK, BACKTICK];
