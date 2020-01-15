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
            conn = pg.connect('dbname=postgres host=localhost port=5000 user=postgres')
            cur = conn.cursor()
            return conn, cur
        except:
            time.sleep(2)
            continue

def hash_password(password):
    hasher = hashlib.sha256()
    hasher.update(str.encode(password))
    return hasher.digest()

# USE GET TO CHECK IF A USER EXIST
# @app.route('/credentials', methods=['GET'])
# def check_credentials():
#     content = request.headers
#     if check_credentials(content):
#         return jsonify({}), 200
#     return jsonify({'message': 'Invalid username or password.'}), 400

# USE POST TO ADD A NEW ACCOUT TO THE DATABASE
# @app.route('/credentials', methods=['POST'])
# def add_user():
#     content = request.headers
#     if check_credentials(content):
#         return jsonify({}), 200
#     return jsonify({'message': 'Invalid username or password.'}), 400

# def check_credentials(content):
#     try:
#         username, password = content['username'], content['password']
#     except:
#         return False
#     dbconn, cur = db_connection()
#     hash = hash_password(password)
#     cur.execute(
#         "SELECT COUNT(*) FROM user_accounts WHERE username = %s AND hashed_password = %s",
#         [username, hash])
#     return cur.fetchone()[0] == 1

@app.route('/', methods=['GET'])
def download():
    return send_from_directory("/", "snake.exe", as_attachment=True)

@app.route('/upload_score/<difficulty>/<username>/<score>', methods=['POST'])
def upload_score():
    conn, cur = db_connection()
    cur.execute("INSERT INTO scores (difficulty, username, score) VALUES (%s, %s, %s)", [difficulty, username, score])
    conn.commit()


@app.route('/scores', methods=['POST']):
def scores():
    conn, cur = db_connection()
    cur.execute("SELECT score, username, data FROM scores ORDER BY score DESC LIMIT 10")
    return json([{'score': row[0], 'username': row[1], 'data': row[2]} for row in cur.fetchall()])

app.run(debug=True, host='0.0.0.0', port=80, threaded=True)
