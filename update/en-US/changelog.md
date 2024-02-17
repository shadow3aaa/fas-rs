# fas-rs(v2.1.8)

[Homepage](https://github.com/shadow3aaa/fas-rs)

## Change log

- Eliminate unnecessary panics

## Running requirements

- No requirements for soc platform
- Android12 or above
- zygisk is enabled and version v2 or above (magisk v24.0 or above and zygisk / othersu + zygisk-next is enabled)

## Special Instructions

- The fas boost mode is a mode specifically used to increase the frame rate. It does not try to limit the maximum frequency, but tries to increase the minimum frequency when stuck to reduce frame drops of the default governor. This mode cannot be judged by the frame rate curve. Is it effective?
- Using shamiko and other hiding methods on games with fas enabled may not take effect. Whether it takes effect depends on whether there is a corresponding game record in `/sdcard/Android/fas-rs/fas_log.txt`.
- Using zygisk injection to hook libgui to obtain frametime, there is some risk of being detected by game.
