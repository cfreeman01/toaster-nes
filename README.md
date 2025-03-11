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

## Game Support
Currently only supports games using mappers 0 or 2. Some example games:

| Mapper | Games |
| -------- | ------- |
| 0 | _Donkey Kong_, _Super Mario Bros._, _Ice Climber_, _Dig Dug_|
| 2 | _DuckTales_, _Mega Man_, _Castlevania_, _Metal Gear_ |

## To-do List
- Add audio
- Support more mappers
- Fix misc. rendering bugs
- Optimize
