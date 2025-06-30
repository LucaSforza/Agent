#!/bin/sh

docker run -it --rm -e DISPLAY=$DISPLAY --net=host --name agent agent ./run.sh $@