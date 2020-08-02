# coding=utf-8
import sys, os, requests, threading
from flask import Flask, render_template, send_file
from flask_qrcode import QRcode

access_token=sys.argv[1]
room_id=sys.argv[2]

body = ""
auth = ""
POOL_TIME = 2

newThread = threading.Thread()

def create_app():
    app = Flask(__name__)
    qrcode = QRcode(app)

    @app.route("/")
    def index():
        return render_template("whatsapp.html", data=body)

    @app.route("/qrcode", methods=["GET"])
    def get_qrcode():
        return send_file(qrcode(body, mode="raw"), mimetype="image/png")

    @app.route("/auth_status")
    def get_auth_status():
        return auth

    def interrupt():
        global newThread
        newThread.cancel()
        os.system('kill %d' % os.getpid())

    def polling():
        global auth
        global body

        if auth == "authorized":
            interrupt()
        else:
            auth = ""

        URL = 'http://localhost:8008/_matrix/client/r0/sync?access_token=' + access_token
        r = requests.get(URL)
        res = r.json()
        events = res["rooms"]["join"][room_id]["timeline"]["events"]
        event = events.pop()
        if "@whatsappbot" in event["sender"]:
            if event["content"]["msgtype"] == "m.image":
                body = event["content"]["body"]
            if event["content"]["msgtype"] == "m.notice":
                if "Successfully logged in" in event["content"]["body"]:
                    auth = "authorized"
            if "m.new_content" in event["content"]:
                if event["content"]["m.new_content"]["msgtype"] == "m.text":
                    if "scan timed out" in event["content"]["body"]:
                        auth = "unauthorized"
        global newThread
        newThread = threading.Timer(POOL_TIME, polling, ())
        newThread.start()

    polling()
    return app

app = create_app()
app.run(host="0.0.0.0", debug=False)
