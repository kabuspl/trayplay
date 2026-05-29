#!/bin/bash

PODIR=$(dirname $0)

xgettext --from-code=UTF-8 -kde -ci18n -ki18n:1 -ki18nc:1c,2 -ki18np:1,2 -ki18ncp:1c,2,3 -ktr2i18n:1 \
    -kI18N_NOOP:1 -kI18N_NOOP2:1c,2 -kaliasLocale -kki18n:1 -kki18nc:1c,2 -kki18np:1,2 -kki18ncp:1c,2,3 \
    $(find . -name \*.qml) -L JavaScript -o "$PODIR/trayplay_qml.pot"

xgettext --from-code=UTF-8 -kde -ci18n -ki18n:1 -ki18nc:1c,2 -ki18np:1,2 -ki18ncp:1c,2,3 -ktr2i18n:1 \
    -kI18N_NOOP:1 -kI18N_NOOP2:1c,2 -kaliasLocale -kki18n:1 -kki18nc:1c,2 -kki18np:1,2 -kki18ncp:1c,2,3 \
    $(find . -name \*.rs) -o "$PODIR/trayplay_rs.pot"

msgcat --use-first "$PODIR/trayplay_qml.pot" "$PODIR/trayplay_rs.pot" > "$PODIR/trayplay.pot"
rm "$PODIR/trayplay_qml.pot" "$PODIR/trayplay_rs.pot"

for cat in $(find . -name '*.po'); do
    echo $cat
    msgmerge -o $cat.new $cat "$PODIR/trayplay.pot"
    mv $cat.new $cat

    LANG_CODE=$(basename $(dirname $cat))
    mkdir -p "locale/$LANG_CODE/LC_MESSAGES"

    msgfmt $cat -o "locale/$LANG_CODE/LC_MESSAGES/trayplay.mo"
done
