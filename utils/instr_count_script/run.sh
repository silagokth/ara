#!/bin/bash
set +x
# get path to bin folder of ara apps from arg
APP=$1
ARA_APPS_BIN_PATH=$2
# if no arg is passed, fail with print usage
if [ -z "$APP" ]; then
    echo "Usage: $0 <ara app name> <path to ara apps bin folder>"
    exit 1
fi
if [ -z "$ARA_APPS_BIN_PATH" ]; then
    echo "Usage: $0 <ara app name> <path to ara apps bin folder>"
    exit 1
fi

if [ "$APP" == "dot32" ]; then
    cargo run -- $ARA_APPS_BIN_PATH/dotproduct.dump dotp_v16b 8 a2 32
elif [ "$APP" == "dot512" ]; then
    cargo run -- $ARA_APPS_BIN_PATH/dotproduct.dump dotp_v16b 8 a2 512
elif [ "$APP" == "2dconv32_3x3" ]; then
    # a0 = block_size_o = 4
    ## a1 = i (input image first address) (DC)
    ## a2 = f_ (DC)
    # a3 = C = 32
    # a4 = F = 3
    # a5 = C + F - 1 = 32 + 3 - 1 = 34
    # a6 = ldo (C << 3) = 32 << 3 = 256
    # a7 = ldi (C + F - 1) << 3 = 272
    # s8 = ldi (C + F - 1) << 3 = 34 << 3 = 272
    # t0 = ldf (F << 3) = 3 << 3 = 24
    cargo run -- $ARA_APPS_BIN_PATH/iconv2d.dump iconv2d_3x3 8 a0 4 a3 32 a4 3 a5 34 a6 256 a7 272 s8 272 t0 24
elif [ "$APP" == "2dconv64_3x3" ]; then
    # a0 = block_size_o = 4
    ## a1 = i (input image first address) (DC)
    ## a2 = f_ (DC)
    # a3 = C = 64
    # a4 = F = 3
    # a5 = C + F - 1 = 64 + 3 - 1 = 66
    # a6 = ldo (C << 3) = 64 << 3 = 512
    # a7 = ldi (C + F - 1) << 3 = 528
    # s8 = ldi (C + F - 1) << 3 = 66 << 3 = 528
    # t0 = ldf (F << 3) = 3 << 3 = 24
    cargo run -- $ARA_APPS_BIN_PATH/iconv2d.dump iconv2d_3x3 8 a0 4 a3 64 a4 3 a5 66 a6 512 a7 528 s8 528 t0 24 
fi
exit 0