# Experimenting with the STM32F3DISCOVERY microcontroller and the Rust programming language

This repo represents my experimentation with using rust on the F3 microcontroller development board.

Besides learning the end goal of the project is reading sensors on the board, sending the readings over bluetooth to a computer where they are visualised.

The approach taken, inspiration and a lot of learning has come from the f3 board support crate found here:
https://github.com/japaric/f3

Currently the system consists of:
- Sensor reading on the microcontroller
- Sensor data serialization and transmission from the microcontroller
- Sensor data receiver and deserialiation on the host PC
- Data visualization on the host PC
