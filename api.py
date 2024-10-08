#!/usr/bin/python3

from flask import Flask, request, jsonify, send_from_directory, render_template
import json
import hashlib
import datetime
import time
import psycopg2 as pg

app = Flask(__name__, template_folder='/render_templates')


def difficulty_in_range(difficulty):
    return 0 <= difficulty and difficulty <= 4


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

@app.route('/available_games', methods=['GET'])
def available_games():
    with open('/available_games.json', 'r') as available_games:
        return jsonify(json.loads(available_games.read()))

@app.route('/', methods=['GET'])
def download_launcher():
    user_agent = request.headers.get('User-Agent')
    if "Windows" in user_agent:
        return send_from_directory("/releases/windows", "launcher.zip", as_attachment=True)
    elif "Linux" in user_agent:
        return send_from_directory("/releases/linux", "launcher.zip", as_attachment=True)
    else:
        return ("Contact us to request support for your system.", 404)


@app.route('/hash/windows', methods=['GET'])
def windows_hash():
    with open('/releases/windows/launcher_hash', 'r') as hash:
        return hash.read()


@app.route('/hash/linux', methods=['GET'])
def linux_hash():
    with open('/releases/linux/launcher_hash', 'r') as hash:
        return hash.read()

# TODO: Fix this, this is just a hack, we can't rely on the snake table.
@app.route('/users')
def users():
    conn, cur = db_connection()
    cur.execute('SELECT DISTINCT username FROM scores')
    return jsonify([row[0] for row in cur.fetchall()])


# Snake


@app.route('/snake', methods=['GET'])
def download_snake():
    user_agent = request.headers.get('User-Agent')
    if "Windows" in user_agent:
        return send_from_directory("/releases/windows", "snake.zip", as_attachment=True)
    elif "Linux" in user_agent:
        return send_from_directory("/releases/linux", "snake.zip", as_attachment=True)
    else:
        return ("Contact us to request support for your system.", 404)


@app.route('/snake/hash/windows', methods=['GET'])
def windows_hash_snake():
    with open('/releases/windows/snake_hash', 'r') as hash:
        return hash.read()


@app.route('/snake/hash/linux', methods=['GET'])
def linux_hash_snake():
    with open('/releases/linux/snake_hash', 'r') as hash:
        return hash.read()


@app.route('/snake/upload_score/<difficulty>/<username>/<score>', methods=['POST'])
def upload_score(difficulty, username, score):
    difficulty = int(difficulty)
    assert difficulty_in_range(difficulty)
    score = int(score)
    assert 0 <= score
    conn, cur = db_connection()
    cur.execute(
        'SELECT MAX(score) FROM scores WHERE difficulty = %s', [difficulty])
    absolute = cur.fetchone()[0]
    cur.execute(
        'SELECT MAX(score) FROM scores WHERE username = %s AND difficulty = %s', [username, difficulty])
    personal = cur.fetchone()[0]
    cur.execute("INSERT INTO scores (difficulty, username, score) VALUES (%s, %s, %s)", [
                difficulty, username, score])
    conn.commit()
    if score > absolute:
        return jsonify({'beaten': 'absolute'}), 201
    if personal is not None:
        if score > personal:
            return jsonify({'beaten': 'personal'}), 201

    return ('', 204)


@app.route('/snake/scores/<difficulty>', methods=['GET'])
def scores_json(difficulty):
    difficulty = int(difficulty)
    assert difficulty_in_range(difficulty)
    _, cur = db_connection()
    cur.execute(
        "SELECT score, username, date FROM scores WHERE difficulty = %s ORDER BY score DESC, date ASC LIMIT 10", [difficulty])
    return jsonify([{'score': row[0], 'username': row[1], 'date': row[2]} for row in cur.fetchall()]), 200


@app.route('/snake/scores_table/<difficulty>', methods=['GET'])
def scores_table(difficulty):
    users = []
    difficulty = int(difficulty)
    assert difficulty_in_range(difficulty)
    _, cur = db_connection()
    cur.execute(
        "SELECT score, username, difficulty FROM scores WHERE difficulty = %s ORDER BY score DESC, date ASC LIMIT 10", [difficulty])
    for row in cur.fetchall():
        users.append({'score': row[0], 'name': row[1], 'difficolta': row[2]})
    return render_template('table.html', users=users)


app.run(debug=True, host='0.0.0.0', port=80, threaded=True)
