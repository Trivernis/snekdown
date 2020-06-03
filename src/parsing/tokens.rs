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
pub(crate) const SPACE: char = ' ';
pub(crate) const EQ: char = '=';
pub(crate) const DOUBLE_QUOTE: char = '"';
pub(crate) const SINGLE_QUOTE: char = '\'';
pub(crate) const DOT: char = '.';
pub(crate) const UP: char = '^';

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
pub(crate) const PHOLDER_OPEN: char = R_BRACKET;
pub(crate) const PHOLDER_CLOSE: char = L_BRACKET;
pub(crate) const CHECK_OPEN: char = R_BRACKET;
pub(crate) const CHECK_CLOSE: char = L_BRACKET;
pub(crate) const CHECK_CHECKED: char = X;

pub(crate) const ITALIC: char = ASTERISK;
pub(crate) const MONOSPACE: char = BACKTICK;
pub(crate) const STRIKED: char = TILDE;
pub(crate) const UNDERLINED: char = UNDERSCR;
pub(crate) const SUPER: char = UP;
pub(crate) const BOLD: [char; 2] = [ASTERISK, ASTERISK];

// groups

pub(crate) const QUOTES: [char; 2] = [SINGLE_QUOTE, DOUBLE_QUOTE];

pub(crate) const BLOCK_SPECIAL_CHARS: [&[char]; 9] = [
    &[HASH],
    &[HASH, META_OPEN],
    &[MINUS, SPACE],
    &SQ_CODE_BLOCK,
    &[PIPE],
    &[QUOTE_START],
    &[META_OPEN],
    &[IMPORT_START, IMPORT_OPEN],
    &SQ_CENTERED_START,
];

pub(crate) const INLINE_SPECIAL_CHARS: [char; 9] = [
    BACKTICK, TILDE, UNDERSCR, ASTERISK, DESC_OPEN, IMG_START, URL_OPEN, LB, SUPER,
];

pub(crate) const LIST_SPECIAL_CHARS: [char; 14] = [
    MINUS, PLUS, ASTERISK, O, '1', '2', '3', '4', '5', '6', '7', '8', '9', '0',
];

// sequences

pub(crate) const SQ_CODE_BLOCK: [char; 3] = [BACKTICK, BACKTICK, BACKTICK];
pub(crate) const SQ_RULER: [char; 5] = [MINUS, SPACE, MINUS, SPACE, MINUS];
pub(crate) const SQ_PHOLDER_START: [char; 2] = [PHOLDER_OPEN, PHOLDER_OPEN];
pub(crate) const SQ_PHOLDER_STOP: [char; 2] = [PHOLDER_CLOSE, PHOLDER_CLOSE];
pub(crate) const SQ_CENTERED_START: [char; 2] = [PIPE, PIPE];

// expressions

pub(crate) const EXPR_URI: &str =
    r"^(https?://)?\w+\.\w+(\.\w+|)?(/[\w, -.%&]+)*/?$|^([\w, -.]+|\w:)?(/[\w, -.]+)+$";
