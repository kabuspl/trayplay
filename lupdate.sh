#!/bin/bash
find ui/lang/ -name "*.ts" -exec /usr/lib/qt6/bin/lupdate ui -ts {} \;
