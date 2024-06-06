@0xbb6ab29910635043;

struct Timestamp {
    seconds @0 :Int64;
    nanos @1 :UInt32;
}

struct Header {
    timestamp @0 :Timestamp;
}

struct Localisation {
    header @0 :Header;

    latitude @1 :Float64;
    longitude @2 :Float64;
    altitude @3 :Float64;
}
