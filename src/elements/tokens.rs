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
pub(crate) const COLON: char = ':';
pub(crate) const PARAGRAPH: char = '§';
pub(crate) const SEMICOLON: char = ';';
pub(crate) const R_BRACE: char = '{';
pub(crate) const L_BRACE: char = '}';
pub(crate) const PERCENT: char = '%';
pub(crate) const COMMA: char = ',';
pub(crate) const MATH: char = '$';
pub(crate) const DOLLAR: char = '$';
pub(crate) const AMPERSAND: char = '&';
pub(crate) const QUESTION_MARK: char = '?';

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
pub(crate) const COLOR_START: char = PARAGRAPH;
pub(crate) const COLOR_OPEN: char = R_BRACKET;
pub(crate) const COLOR_CLOSE: char = L_BRACKET;
pub(crate) const BIBREF_OPEN: char = R_BRACKET;
pub(crate) const BIBREF_REF: char = UP;
pub(crate) const BIBREF_CLOSE: char = L_BRACKET;
pub(crate) const BIB_KEY_OPEN: char = R_BRACKET;
pub(crate) const BIB_KEY_CLOSE: char = L_BRACKET;
pub(crate) const BIB_DATA_START: char = COLON;
pub(crate) const TEMP_VAR_OPEN: char = R_BRACE;
pub(crate) const TEMP_VAR_CLOSE: char = L_BRACE;
pub(crate) const TEMPLATE: char = PERCENT;

pub(crate) const ITALIC: char = ASTERISK;
pub(crate) const MONOSPACE: char = BACKTICK;
pub(crate) const STRIKED: &'static [char] = &[TILDE, TILDE];
pub(crate) const UNDERLINED: char = UNDERSCR;
pub(crate) const SUPER: char = UP;
pub(crate) const EMOJI: char = COLON;
pub(crate) const MATH_INLINE: &'static [char] = &[MATH, MATH];
pub(crate) const BOLD: &'static [char] = &[ASTERISK, ASTERISK];

pub(crate) const CHARACTER_START: char = AMPERSAND;
pub(crate) const CHARACTER_STOP: char = SEMICOLON;

pub(crate) const GLOSSARY_REF_START: char = TILDE;

// Reference Anchors

pub(crate) const ANCHOR_START: &'static [char] = &[R_BRACKET, QUESTION_MARK];
pub(crate) const ANCHOR_STOP: char = L_BRACKET;

// References

pub(crate) const REF_START: &'static [char] = &[R_BRACKET, DOLLAR];
pub(crate) const REF_STOP: char = L_BRACKET;
pub(crate) const REF_DESC_START: char = R_PARENTH;
pub(crate) const REF_DESC_STOP: char = L_PARENTH;

// Arrows

pub(crate) const A_RIGHT_ARROW: &'static [char] = &['-', '-', '>'];
pub(crate) const A_LEFT_ARROW: &'static [char] = &['<', '-', '-'];
pub(crate) const A_LEFT_RIGHT_ARROW: &'static [char] = &['<', '-', '-', '>'];
pub(crate) const A_BIG_RIGHT_ARROW: &'static [char] = &['=', '=', '>'];
pub(crate) const A_BIG_LEFT_ARROW: &'static [char] = &['<', '=', '='];
pub(crate) const A_BIG_LEFT_RIGHT_ARROW: &'static [char] = &['<', '=', '=', '>'];

// groups

pub(crate) const QUOTES: [char; 2] = [SINGLE_QUOTE, DOUBLE_QUOTE];

pub(crate) const BLOCK_SPECIAL_CHARS: &'static [&[char]] = &[
    &[HASH],
    &[HASH, META_OPEN],
    &[MINUS, SPACE],
    &SQ_CODE_BLOCK,
    &[PIPE],
    &[QUOTE_START],
    &[META_OPEN],
    &[IMPORT_START, IMPORT_OPEN],
    &SQ_CENTERED_START,
    &SQ_MATH,
];

pub(crate) const INLINE_SPECIAL_CHARS: &'static [char] = &[
    BACKTICK,
    TILDE,
    UNDERSCR,
    ASTERISK,
    DESC_OPEN,
    IMG_START,
    URL_OPEN,
    LB,
    SUPER,
    EMOJI,
    COLOR_START,
    MATH,
];

pub(crate) const INLINE_SPECIAL_SEQUENCES: &'static [&'static [char]] = &[
    A_BIG_LEFT_RIGHT_ARROW,
    A_BIG_LEFT_ARROW,
    A_BIG_RIGHT_ARROW,
    A_RIGHT_ARROW,
    A_LEFT_ARROW,
    A_LEFT_RIGHT_ARROW,
    ANCHOR_START,
    REF_START,
];

pub(crate) const LIST_SPECIAL_CHARS: [char; 14] = [
    MINUS, PLUS, ASTERISK, O, '1', '2', '3', '4', '5', '6', '7', '8', '9', '0',
];

pub(crate) const WHITESPACE: &[char] = &[' ', '\t', '\r', '\n'];
pub(crate) const INLINE_WHITESPACE: [char; 3] = [' ', '\t', '\r'];

// sequences

pub(crate) const SQ_CODE_BLOCK: [char; 3] = [BACKTICK, BACKTICK, BACKTICK];
pub(crate) const SQ_RULER: [char; 5] = [MINUS, SPACE, MINUS, SPACE, MINUS];
pub(crate) const SQ_PHOLDER_START: [char; 2] = [PHOLDER_OPEN, PHOLDER_OPEN];
pub(crate) const SQ_PHOLDER_STOP: [char; 2] = [PHOLDER_CLOSE, PHOLDER_CLOSE];
pub(crate) const SQ_CENTERED_START: [char; 2] = [PIPE, PIPE];
pub(crate) const SQ_COLOR_START: [char; 2] = [COLOR_START, COLOR_OPEN];
pub(crate) const SQ_BIBREF_START: [char; 2] = [BIBREF_OPEN, BIBREF_REF];
pub(crate) const SQ_MATH: &'static [char] = &[MATH, MATH, MATH];
