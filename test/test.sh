#!/bin/bash
set -euxo

extensions=("mp3" "m4a" "flac" "ogg" "wav" "aac" "m4b" "oga" "opus" "webm")


# Test 1: estensioni
for i in ${extensions[@]};
do
    cargo run -- -i "$1" -o "$2"/test1 -f "$i" 
done
echo "Test 1 eseguito"
notify-send "Test 1 eseguito" -u normal
read -n 1 -p "Premi invio per continuare:" pausa


# Test 2: niente sovrascrittura
for i in ${extensions[@]};
do
    cargo run -- -i "$1" -o "$2"/test1 -f "$i"
done
echo "Test 2 eseguito"
notify-send "Test 2 eseguito" -u normal
read -n 1 -p "Premi invio per continuare:" pausa


# Test 3: sovrascrittura
for i in ${extensions[@]};
do
    cargo run -- -i "$1" -o "$2"/test1 -f "$i" -s
done
echo "Test 3 eseguito"
notify-send "Test 3 eseguito" -u normal
read -n 1 -p "Premi invio per continuare:" pausa
rm -rv "$2"/test1


# Test uso di thread impostati via argomento
echo "Il numero di thread aspettato è di $(($(nproc)/2))"
for i in ${extensions[@]};
do
    cargo run -- -i "$1" -o "$2"/test4 -f "$i" -t $(($(nproc)/2))
done
echo "Test 4 eseguito"
notify-send "Test 4 eseguito" -u normal
read -n 1 -p "Premi invio per continuare:" pausa

# Test uso di thread impostati via argomento con sovrascrittura
echo "Il numero di thread aspettato è di $(($(nproc)/2))"
for i in ${extensions[@]};
do
    cargo run -- -i "$1" -o "$2"/test4 -f "$i" -t $(($(nproc)/2)) -s
done
echo "Test 5 eseguito"
notify-send "Test 5 eseguito" -u normal
read -n 1 -p "Premi invio per continuare:" pausa
rm -rv "$2"/test4