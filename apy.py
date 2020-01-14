from flask import Flask, request, jsonify
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

@app.route('/')
def root():
    return jsonify({
        'message': 'Call an endpoint to interact with the API.',
        'endpoints': []
    }), 404

def hash_password(password):
    hasher = hashlib.sha256()
    hasher.update(str.encode(password))
    return hasher.digest()

# USE GET TO CHECK IF A USER EXIST
@app.route('/credentials', methods=['GET'])
def check_credentials():
    content = request.headers
    if check_credentials(content):
        return jsonify({}), 200
    return jsonify({'message': 'Invalid username or password.'}), 400

# USE POST TO ADD A NEW ACCOUT TO THE DATABASE
@app.route('/credentials', methods=['POST'])
def add_user():
    content = request.headers
    if check_credentials(content):
        return jsonify({}), 200
    return jsonify({'message': 'Invalid username or password.'}), 400

def check_credentials(content):
    try:
        username, password = content['username'], content['password']
    except:
        return False
    dbconn, cur = db_connection()
    hash = hash_password(password)
    cur.execute(
        "SELECT COUNT(*) FROM user_accounts WHERE username = %s AND hashed_password = %s",
        [username, hash])
    return cur.fetchone()[0] == 1