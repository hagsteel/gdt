extends Node

var gdn

func _ready():
    print('(GD) Testing...')
    gdn = GDNative.new()
    var status = false

    gdn.library = load("res://lib{{name}}.gdnlib")

    if gdn.initialize():
        status = gdn.call_native("standard_varcall", "run_tests", [])

        if status:
            print('all tests passed')
        else:
            print('test failure')

        gdn.terminate()
        get_tree().quit(1)
