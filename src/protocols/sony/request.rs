/*
Protocol note
- almost everything is start with 3e
- packet with UID contain no data
- This payload: [3e 01 00 00 00 00 00 01 3c] is this a fucking ACK. i saw it like every 10 packet
- The headphone never concatinate ACK with other payload but my phone (maybe the sony app) do

- This payload: [3e 01 01 00 00 00 00 02 3c] appear a lot too
    - i think this is ack after the headphone sent something to the app
    - it will repeat the message until ack
- i didnt see any [3e 01 {02..ff} ...]

Founded: [3e {01, 0c, 0e} ...]


*/