# ToasterNES
Simple NES emulator written in Rust. Currently in development.

![image](https://github.com/user-attachments/assets/4c250521-6fe3-4fc2-9bdc-034085250a2d)
![image](https://github.com/user-attachments/assets/efa86d88-5593-4bec-abbd-7908a80c7568)

## Building
TODO: add build instructions for Linux and Windows

## Running
Simply run from the command line, passing in the path to the ROM as argument: 

`cargo run --release ~/ROMs/NES/Donkey\ Kong/Donkey\ Kong\ \(World\)\ \(Rev\ 1\).nes`

Key controls are currently hardcoded:
| Key | Button |
| -------- | ------- |
| WASD | Up, Left, Down, Right |
| Q | Select |
| E | Start |
| L | A |
| K | B |

## Missing Features/To-do
- No audio support
- Only supports mapper 0 ROMs
- Need to make keys re-bindable, possibly support gamepad input
- Need to add support for 8x16 sprites, other misc. PPU features
