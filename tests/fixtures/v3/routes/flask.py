# Flask route fixtures
from flask import Flask

app = Flask(__name__)

@app.route('/users', methods=['GET'])
def list_users():
    return []

@app.route('/users/<id>', methods=['GET', 'PUT'])
def user(id):
    return {"id": id}

@app.route('/users', methods=['POST'])
def create_user():
    return {}, 201
