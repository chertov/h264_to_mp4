ffmpeg -framerate 25 -i stream_chn0.h264 -c copy -f mp4 -movflags frag_keyframe+empty_moov output.mp4
