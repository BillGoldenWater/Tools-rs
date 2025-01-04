ffmpeg -y -i input.mp4 -map 0:a:0 -ar 44100 -ac 1 -f f64le audio.bin

rm -r ./output
mkdir output

cargo run --release

rm audio.bin
