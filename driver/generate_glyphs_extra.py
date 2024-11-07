from typing import IO

input_filenames = ['./glyphs_extra.txt']
output_filename = './src/renderer/glyphs/generated_extra.rs'

empty_characters = '. '

file_template_start = '''use std::collections::HashMap;

use lazy_static::lazy_static;

use super::Glyph;

lazy_static! {
    pub static ref CHARS_EXTRA: HashMap<char, Glyph> = chars();
}

pub fn chars() -> HashMap<char, Glyph> {
    let mut m = HashMap::new();

'''

file_template_end = '''
    m
}
'''

line_template = "    m.insert('{char}', 0b{value:0>64b}.into());\n"

# dicts in python are guaranteed to enumerate in insertion order
charmap = {}

line_number = 0

def generate():
    global line_number

    for input_filename in input_filenames:
        line_number = 0

        with open(input_filename, encoding='utf-8') as input_file:
            while True:
                line_number += 1
                chars = read_characters(input_file)
                if len(chars) == 0:
                    break

                value = parse_glyph(input_file)
                for c in chars:
                    if c in charmap:
                        raise ValueError(f"Line {line_number}: character '{c}' already exists")
                    charmap[c] = value
                    # print(f"registered '{c}'")

    with open(output_filename, 'w', encoding='utf-8') as output_file:
        output_file.write(file_template_start)
        output_file.writelines(line_template.format(char=escaped(c), value=v) for c, v in charmap.items())
        output_file.write(file_template_end)

def escaped(c: str) -> str:
    if c == '\'':
        return '\\\''

    if c == '\\':
        return '\\\\'

    return c

def read_characters(file: IO[str]) -> str:
    chars = ''

    next_char = file.read(1)
    if len(next_char) == 0:
        return ''
    if next_char == '\n':
        raise ValueError(f'Line {line_number}: expected characters, found empty line')
    while next_char != '\n':
        chars += next_char
        next_char = file.read(1)
    return decode_unicode(chars)

def decode_unicode(chars: str) -> str:
    # print(chars)
    if chars.startswith('U+'):
        try:
            code_point = int(chars[2:], 16)
            return chr(code_point)
        except:
            raise ValueError(f'Line {line_number}: could not decode {chars} as unicode character')

    return chars

def parse_glyph(file: IO[str]) -> int:
    global line_number

    glyph_data = read_lines(file, 8)
    if glyph_data[7] != '':
        raise ValueError(f"Line {line_number + 8}: expected blank line, found '{glyph_data[7]}'")

    line_length = len(glyph_data[0])
    if line_length > 9:
        raise ValueError(f'Line {line_number + 1}: width of glyph exceeds maximum width of 9 (actually {line_length})')
    if line_length == 0:
        raise ValueError(f'Line {line_number + 1}')

    value = 0
    for i in range(7):
        line = glyph_data[i]
        if len(line) != line_length:
            raise ValueError(f'Line {line_number + i + 1}: this glyph is {line_length} wide (as defined on line {line_number + 1}), but this line is not {line_length} characters long (actually {len(line)})')

        for char in line:
            bit_value = 0 if char in empty_characters else 1
            value += bit_value
            value <<= 1

    # reverse the last bit shift, then store the line length (see definition of Glyph for details)
    value >>= 1
    value |= (line_length - 1) << 60

    line_number += 8
    return value

def read_lines(file: IO[str], lines: int) -> list[str]:
    result = []
    for _ in range(lines):
        result.append(file.readline().rstrip(' \n'))
    return result

if __name__ == '__main__':
    generate()
