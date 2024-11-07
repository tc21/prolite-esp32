from PIL import Image, ImageFont, ImageDraw

font_file = r"C:\Users\lanxia\Downloads\fusion-pixel-font-8px-monospaced-ttf-v2024.11.04\fusion-pixel-8px-monospaced-ja.ttf"
font = ImageFont.truetype(font_file, 8)
glyph_size = (7, 7)
glyph_offset = (0, -1)
full_width = True

def main():
    unknown_glyph = to_glyph('🦊')

    with open('glyphs_from_font.txt', 'w', encoding='utf-8') as out_file:
        for i in range(0x3040, 0x110000):
            c = chr(i)
            glyph = to_glyph(c)
            if glyph != unknown_glyph:
                out_file.write(f'{c}\n')
                for line in glyph:
                    out_file.write(''.join(line) + '\n')
                out_file.write('\n')

            if i % 1000 == 0:
                print(i)

def to_glyph(char: str) -> list[list[str]]:
    image = Image.new("1", glyph_size, 0)
    draw = ImageDraw.Draw(image)
    draw.text(glyph_offset, char, font=font, fill=1)
    # image.save('out.png')
    # exit()
    return image_to_glyph(image)

def image_to_glyph(image: Image) -> list[list[str]]:
    glyph = [['.'] * 7 for _ in range(7)]
    for x in range(7):
        for y in range(7):
            pixel = image.getpixel((x, y))
            if pixel == 1:
                glyph[y][x] = 'x'

    remove = []
    for x in range(7):
        if all(glyph[y][x] == '.' for y in range(7)):
            remove.append(x)
        else:
            break

    if not full_width:
        start = (1 + max(remove)) if len(remove) > 0 else 0
        for x in reversed(range(start, 7)):
            if all(glyph[y][x] == '.' for y in range(7)):
                remove.append(x)
            else:
                break

        for x in reversed(sorted(remove)):
            for y in range(7):
                del glyph[y][x]

    return glyph

main()
