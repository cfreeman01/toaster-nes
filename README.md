# ToasterNES
Simple NES emulator written in Rust. Currently in development.

![image](https://github.com/user-attachments/assets/68f7cadc-ddb7-43a6-b382-a7b103b8be50)
![image](https://github.com/user-attachments/assets/06bc51a1-b8fa-41fa-853f-bf6caf7b3842)
![image](https://github.com/user-attachments/assets/474ce6fd-89a1-430f-a710-0a0b5b4f8464)
![image](https://github.com/user-attachments/assets/78a635c8-018f-4a93-82c0-388aa84fd68c)

## Building
TODO: add build instructions for Linux and Windows

## Running
Run from the command line, passing in the path to the ROM as argument: 

`cargo run --release <ROM path>`

To play the game, use the keyboard controls:
| Key | Button |
| -------- | ------- |
| WASD | Up, Left, Down, Right |
| Q | Select |
| E | Start |
| L | A |
| K | B |

## Mapper Support
"Mappers" represent different types of NES cartridges. 

Currently supported mappers:

| Mapper | Example Games |
| -------- | ------- |
| 0 | _Donkey Kong_, _Super Mario Bros._, _Ice Climber_, _Dig Dug_|
| 2 | _DuckTales_, _Mega Man_, _Castlevania_, _Metal Gear_ |

## To-do List
- Add audio
- Support more mappers
