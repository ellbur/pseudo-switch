
`pseudo-switch` is a program to emulate a tablet-mode switch for Linux devices with detachable keyboards.

`pseudo-switch` monitors for the keyboard being added or removed. When the keyboard is removed, it turns the tablet-mode switch ON, and when the keyboard is added, it turns the table-mode switch OFF.

`pseudo-switch` can be used, for example, with Sway's `bindswitch tablet:on`.

# Building

```
cargo build --release
```

# Usage

## Identifying the Correct Device

First, identify the device for your detachable keyboard. Run:

```
pseudo-switch identify-detachable-devices
```

then try removing the detachable keyboard. The correct device should become grayed out.

It is recommended to use the path under `/dev/input/by-path` since it will not change regardless of the order in which devices are added.

## Permissions

The user under which `pseudo-switch` is run must have write permissions for `/dev/uinput`.

Here's how you can check your permissions:

```
$ ls -l /dev/uinput
crw-rw----+ 1 root input 10, 223 Mar 12 22:37 /dev/uinput

$ groups
sys network power lp input
```

## Running the Emulated Tablet-Mode Switch

```
pseudo-switch run /dev/input/by-path/<your-device>
```

For example, on my system, it would be:

```
pseudo-switch run /dev/input/by-path/pci-0000:00:14.0-usbv2-0:3.1:1.0-event-kbd  
```

You may wish to add hysteresis (a delay) to prevent rapid oscillation. For example, to add a 0.5 second delay:

```
pseudo-switch run --hysteresis 0.5 /dev/input/by-path/<your-device>
```

