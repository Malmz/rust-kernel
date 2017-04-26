#!/bin/bash
# Vscode tasks are stupid, i have to use gnome terminal for vscode not lock up
gnome-terminal -e "nohup make $1 &"
