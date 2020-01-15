FROM ubuntu:latest
RUN apt update -y && \
apt install ca-certificates libssl-dev libpq-dev python3 python3-pip -y
COPY requirements.txt .
RUN python3 -m pip install -r requirements.txt
COPY api.py /usr/bin
COPY target/x86_64-pc-windows-gnu/release/snake.exe /
CMD ["python3", "/usr/bin/api.py"]