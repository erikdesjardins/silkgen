# silkgen

Generate KiCad silkscreen art from PNGs.

Primarily intended for pixel art. Turns black/white/transparent pixels into copper/silkscreen/nothing respectively.

Adds clearance between silkscreen and copper to avoid DRC violations.

## Example

`silkgen --pixel-pitch 0.5mm --clearance 0.05mm annoying_dog.png -o annoying_dog.kicad_mod`

| annoying_dog.png | annoying_dog.kicad_mod |
| ------------- | ------------- |
| <img width="644" alt="example_image" src="https://user-images.githubusercontent.com/7673145/177067376-68128023-bc9b-4744-ada7-9dcbf2dfdf09.png"> | <img width="723" alt="example_footprint" src="https://user-images.githubusercontent.com/7673145/177067390-aa2a6aaa-5c40-4e39-8d5f-1cd2400b97ad.png"> |
