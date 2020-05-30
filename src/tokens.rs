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
pub(crate) const GT: char = '>';
pub(crate) const LT: char = '<';
pub(crate) const BANG: char = '!';

// aliases

pub(crate) const SPECIAL_ESCAPE: char = BACKSLASH;
pub(crate) const META_OPEN: char = R_BRACKET;
pub(crate) const META_CLOSE: char = L_BRACKET;
pub(crate) const QUOTE_START: char = GT;
pub(crate) const DESC_OPEN: char = R_BRACKET;
pub(crate) const DESC_CLOSE: char = L_BRACKET;
pub(crate) const IMG_START: char = BANG;
pub(crate) const URL_OPEN: char = R_PARENTH;
pub(crate) const URL_CLOSE: char = L_PARENTH;
pub(crate) const IMPORT_START: char = LT;
pub(crate) const IMPORT_OPEN: char = R_BRACKET;
pub(crate) const IMPORT_CLOSE: char = L_BRACKET;

// groups

pub(crate) const BLOCK_SPECIAL_CHARS: [char; 7] = [
    HASH,
    MINUS,
    BACKTICK,
    PIPE,
    QUOTE_START,
    META_OPEN,
    IMPORT_START,
];
pub(crate) const INLINE_SPECIAL_CHARS: [char; 6] = [LB, ASTERISK, UNDERSCR, TILDE, PIPE, BACKTICK];
pub(crate) const INLINE_SPECIAL_CHARS_SECOND: [char; 3] = [DESC_OPEN, IMG_START, URL_OPEN];

pub(crate) const LIST_SPECIAL_CHARS: [char; 4] = [MINUS, PLUS, ASTERISK, O];

// sequences

pub(crate) const SQ_CODE_BLOCK: [char; 3] = [BACKTICK, BACKTICK, BACKTICK];

// expressions

pub(crate) const EXPR_URI: &str =
    r"^(https?://)?\w+\.\w+(\.\w+|)?(/[\w, -.%&]+)*/?$|^([\w, -.]+|\w:)?(/[\w, -.]+)+$";
