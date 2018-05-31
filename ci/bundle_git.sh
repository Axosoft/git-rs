#!/bin/bash

if [ $TARGET == 'x86_64-apple-darwin' ]; then
    DOWNLOAD_URL='https://github.com/desktop/dugite-native/releases/download/v2.17.1/dugite-native-v2.17.1-macOS.tar.gz'
elif [ $TARGET == 'x86_64-unknown-linux-gnu' ]; then
    DOWNLOAD_URL='https://github.com/desktop/dugite-native/releases/download/v2.17.1/dugite-native-v2.17.1-ubuntu.tar.gz'
fi

curl -L --output vendor.tar.gz $DOWNLOAD_URL
mkdir vendor
tar -xzvf vendor.tar.gz -C vendor