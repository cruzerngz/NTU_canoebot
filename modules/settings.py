'''Settings for canoebot. Do not modify this file.
To change settings see botsettings.json'''
import json as jsn
import lib.Dotionary as dot

## path to settings file - do not change!
## modify the json file to change settings
# _path = './.configs/botsettings.json' ## deployed version
_path = './.configs/botsettings.debug.json' ## debug version

with open(_path) as jsonfile:
    json = jsn.load(jsonfile)

json = dot.to_dotionary(json)
