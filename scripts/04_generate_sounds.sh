#!/bin/sh
python3 ./scripts/04_generate_sounds.py
for w in tmp/game_sfx/*.wav; do
    k=${w#tmp\/game_sfx/}; ffmpeg -i "$w" ${k%.wav}.ogg;
done

