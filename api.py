#!/usr/bin/python3

from flask import Flask, request, jsonify, send_from_directory
import json
import hashlib
import datetime
import time
import psycopg2 as pg

app = Flask(__name__)


def db_connection():
    while True:
        try:
            conn = pg.connect(
                'dbname=postgres host=localhost port=6000 user=postgres')
            cur = conn.cursor()
            return conn, cur
        except:
            time.sleep(2)
            continue


def hash_password(password):
    hasher = hashlib.sha256()
    hasher.update(str.encode(password))
    return hasher.digest()


@app.route('/', methods=['GET'])
def download():
    return send_from_directory("/", "snake.zip", as_attachment=True)


@app.route('/upload_score/<difficulty>/<username>/<score>', methods=['POST'])
def upload_score(difficulty, username, score):
    difficulty = int(difficulty)
    assert 0 <= difficulty and difficulty <= 4
    score = int(score)
    assert 0 <= score
    conn, cur = db_connection()
    cur.execute("INSERT INTO scores (difficulty, username, score) VALUES (%s, %s, %s)", [
                difficulty, username, score])
    conn.commit()
    return ('', 204)


@app.route('/scores/<difficulty>', methods=['GET'])
def scores(difficulty):
    difficulty = int(difficulty)
    assert 0 <= difficulty and difficulty <= 4
    conn, cur = db_connection()
    cur.execute(
        "SELECT score, username, date FROM scores WHERE difficulty = %s ORDER BY score DESC, date ASC LIMIT 10", [difficulty])
    return jsonify([{'score': row[0], 'username': row[1], 'date': row[2]} for row in cur.fetchall()]), 201

@app.route('/absolute_and_personal_high_score/<difficulty>/<username>', methods=['GET'])
def absolute_and_personal_high_score(difficulty, username):
    difficulty = int(difficulty)
    assert 0 <= difficulty and difficulty <= 4
    conn, cur = db_connection()
    cur.execute('SELECT MAX(score) FROM scores')
    absolute = cur.fetchone()[0]
    cur.execute('SELECT MAX(score) FROM scores WHERE username = %s', [username])
    personal = cur.fetchone()[0]
    return jsonify({'absolute': absolute, 'personal': personal}), 200

app.run(debug=True, host='0.0.0.0', port=80, threaded=True)
