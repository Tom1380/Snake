FROM ubuntu:latest
RUN apt update -y && \
apt install ca-certificates libssl-dev libpq-dev python3 python3-pip zip -y
COPY requirements.txt .
RUN python3 -m pip install -r requirements.txt
COPY api.py /usr/bin
WORKDIR /
RUN mkdir -p  releases/windows
RUN mkdir releases/linux
COPY target/x86_64-pc-windows-gnu/release/snake.exe /releases/windows
COPY target/release/snake /releases/linux
RUN cd /releases/windows && zip snake.zip snake.exe
RUN cd /releases/linux && zip snake.zip snake
COPY render_templates /render_templates
CMD ["python3", "/usr/bin/api.py"]