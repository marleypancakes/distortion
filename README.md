# Foldback Distortion Effect

This is a VST3 plugin that does written in Rust using the open source [nih-plug](https://github.com/robbert-vdh/nih-plug) crate. It can be compiled as a VST for use with digital audio workstations or as a standalone application.

The plugin currently has two sliders: 
-Threshold, which will apply distortion by inverting audio signals over the specified decibel value 
-Mix, which controls how much of the distorted signal is mixed into the original clean audio stream

After installing [Rust](https://rustup.rs/), you can compile the plugin as follows:

```shell
cargo xtask bundle distortion --release
```

Currently, this builds the plugin as both standalone and VST3. The standalone plugin is then ready to go, while the VST version must be imported into a DAW.

## Screenshots

![image](https://github.com/marleypancakes/distortion/assets/82685635/76d5ee79-59e0-41d6-85cb-d0c84df5a69c)
