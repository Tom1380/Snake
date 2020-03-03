FROM ubuntu:latest
RUN apt update -y && \
apt install ca-certificates libssl-dev libpq-dev python3 python3-pip zip -y
COPY requirements.txt .
RUN python3 -m pip install -r requirements.txt
COPY api.py /usr/bin
WORKDIR /
COPY target/x86_64-pc-windows-gnu/release/snake.exe /
RUN zip snake.zip snake.exe
COPY render_templates /render_templates
CMD ["python3", "/usr/bin/api.py"]