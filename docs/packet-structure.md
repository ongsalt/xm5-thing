# Packet structure

## Infomation Soruce
- [SonyHeadphonesClient](https://github.com/mos9527/SonyHeadphonesClient)
- [GadgetBridge](https://github.com/Freeyourgadget/Gadgetbridge/blob/master/app/src/main/java/nodomain/freeyourgadget/gadgetbridge/service/devices/sony/headphones/protocol/impl/v3/SonyProtocolImplV3.java)


## From Headphone
1 bytes each unless otherwised marked
- START_MARKER, dataType, seqNumber, size (u32?), checksum, ...payload, END_MARKER



## To Headphone
