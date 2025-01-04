# ipad
ffmpeg -y \
    -i ./input.mp4 \
    -r 60 -i output/%10d.png \
    -filter_complex "[0:v][1:v] overlay=180:1300" \
    -c:a copy -c:v libx264 -crf 23 -preset ultrafast \
    output.mp4

# pc
# ffmpeg -y \
#     -i ./input.mp4 \
#     -r 60 -i output/%10d.png \
#     -filter_complex "[0:v][1:v] overlay=190:950" \
#     -c:a copy -c:v libx264 -crf 23 -preset ultrafast \
#     output.mp4
