FROM ubuntu:latest
RUN apt update -y && \
apt install ca-certificates libssl-dev libpq-dev python3 python3-pip zip -y
COPY requirements.txt .
RUN python3 -m pip install -r requirements.txt
COPY api.py /usr/bin
WORKDIR /
COPY available_games.json .
RUN mkdir -p  releases/windows
RUN mkdir releases/linux
COPY target/x86_64-pc-windows-gnu/release/launcher.exe /releases/windows
COPY target/x86_64-pc-windows-gnu/release/snake.exe /releases/windows
COPY target/release/launcher /releases/linux
COPY target/release/snake /releases/linux
RUN cd /releases/windows && zip launcher.zip launcher.exe && sha256sum launcher.exe | awk '{ print $1 }' > launcher_hash
RUN cd /releases/windows && zip snake.zip snake.exe && sha256sum snake.exe | awk '{ print $1 }' > snake_hash
RUN cd /releases/linux && zip launcher.zip launcher && sha256sum launcher | awk '{ print $1 }' > launcher_hash
RUN cd /releases/linux && zip snake.zip snake && sha256sum snake | awk '{ print $1 }' > snake_hash
COPY render_templates /render_templates
CMD ["python3", "/usr/bin/api.py"]