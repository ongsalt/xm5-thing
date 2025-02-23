# Bluetooth packet capturing

## Using android devices
1. turn on dev mode
2. enable adb
3. enable bluetooth hci snoop log
4. toggle bluetooth (you might need to reboot your device if you are on older version of android)
5. connect your phone to your pc via a usb cable or wireless debugging (your pc should have adb installed. if not please download it from [here](https://developer.android.com/tools/adb))
6. [follow the official docs](https://source.android.com/docs/core/connect/bluetooth/verifying_debugging#debugging-options)


### Live capturing
warning: this might break a lot
1. do step 1-5 from previous section
2. open wireshark on your pc
3. you should see `Android Bluetooth Btsoop...` under capture section. double click it.


> if you dont see any packet you might need to connect to the bluetooth device after doing steps above 