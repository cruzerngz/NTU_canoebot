'''This is the main file that gets called on canoebot startup. Sparse eh?'''

from bot_modules.common_core import CanoeBot as bot
import bot_modules

import lib.liblog as lg

lg.functions.info('starting canoebot...')

## run any set events on startup
bot_modules.events.init()

bot.infinity_polling()#timeout=10, long_polling_timeout=5)
