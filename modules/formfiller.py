'''Interface to SCF for submitting daily log sheet'''

import requests as rq
from datetime import date
from dateutil.parser import parse
import random

## bot modules
import lib.liblog as lg

import modules.sheetscraper as ss
import lib.gForm as gf
import modules.settings as s

lg.functions.debug("formfiller loaded")

## change the form id if needed (should be never)
## if SCF suddenly decides to change their form, then formfiller will need to be updated
FORM_ID = s.json.formfiller.form_id
url = f'https://docs.google.com/forms/d/e/{FORM_ID}/formResponse'

## if the exco changes then pls fill in new details
## dictionary is in this format:
## key:     [str your_name]
## value:   [str your_8_digit_hp_number]
PARTICULARS = s.json.formfiller.particulars

class logSheet():
    def __init__(self):
        self.name       = None  ## key in the particulars dictionary
        self.contact    = None  ## value in the particulars dictionary
        self.star1      = None  ## number of 1 star people
        self.star0      = None  ## number of 0 star people
        self.date       = None  ## date in datetime.date format
        self.datestr    = None  ## date in string form
        self.timeslot   = 0     ## AM or PM timeslot, enum(0,1), default 0
        self.isoday     = None  ## day represented as a number
        self.ispresent  = None  ## if log sheet day is same as current day
        self.starttime  = None  ## earliest-boat-in-water time (estimate)
        self.endtime    = None  ## latest-boat-out-of-water time (also estimate)
        self.form       = None  ## Constructed form
        self.gForm      = gf.gForm(FORM_ID)

    ## modifies name and contact
    def getparticulars(self):
        self.name,self.contact = random.choice(list(PARTICULARS.items()))

    def __certstatus(self):
        return ss.CERT_STATUS

    def __getnamelist(self):
        timeslot = 'am' if self.timeslot==0 else 'pm'
        df = ss.namelist(f'{self.datestr}, {timeslot}')[3:]
        return df.reset_index(drop=True)

    ## modifies star1 and star0
    def getcertstatus(self):
        df = self.__getnamelist()
        count = len(df)
        certstatus = self.__certstatus() ## get the dictionary

        for i in range(count):
            df.iloc[i] = certstatus[df.iloc[i]] ## replace with 1s and 0s

        self.star1 = df.sum()
        self.star0 = count - self.star1

    ## modifies date, datestr, ispresent and isoday
    def getdate(self, date_in):
        try:
            d = parse(date_in).date()
        except:
            d = date.today()

        if d != date.today():
            self.ispresent = 0
        else:
            self.ispresent = 1

        self.date = d
        self.datestr = d.strftime('%Y-%m-%d')
        self.isoday = d.isoweekday()

    ## tells formfiller which time slot to pull through sheetscraper
    def settimeslot(self, timeslot:int):
        self.timeslot = timeslot

    ## modifies starttime and endtime
    ## no logic added yet
    def gettimes(self):
        if self.timeslot == 0:
            if self.isoday == 7:
                self.starttime = '07:30'
                self.endtime = '10:00'
            else:
                self.starttime = '07:30'
                self.endtime = '09:30'

        elif self.timeslot == 1:
            self.starttime = '16:00'
            self.endtime = '18:00'

    ## Change the number of people present for a session
    def changeattendance(self, new_count):
        self.star1 = new_count
        self.star0 = 0

    ## Change the name
    def changename(self, new_name):
        self.name = new_name

    def generateform(self, date_str=''):
        self.getdate(date_str)
        self.getparticulars()
        self.getcertstatus()
        self.gettimes()

        ## gForm used to fill form, removes the need to inspect element to get entry id
        self.gForm.fill_str(0, self.name)
        self.gForm.fill_str(1, self.contact)
        self.gForm.fill_str(2, "Nanyang Technological University")
        self.gForm.fill_option(3, 1)
        self.gForm.fill_int(4, self.star1)
        self.gForm.fill_int(5, self.star0)
        self.gForm.fill_option(6, 1)
        self.gForm.fill_date(7, self.date)
        self.gForm.fill_str(8, self.starttime)
        self.gForm.fill_str(9, self.endtime)
        self.gForm.fill_option(10, 0)

        sheetfields = {
            ## Name
            'entry.650249987':self.name,
            ## Contact number
            'entry.159891337':self.contact,
            ## Org
            'entry.1940228710':'Nanyang Technological University',
            ## Activity type
            'entry.1965888248':'Co-Curricular Activities (CCA)',
            ## No. of 1 star people
            'entry.1522705696':self.star1,
            ## No. of 0 star people
            'entry.923232455':self.star0,
            ## Training location
            'entry.1917318237':'The Paddle Lodge @ MacRitchie Reservoir',
            ## Date of training
            'entry.2082654990':self.datestr,
            ## Start time
            'entry.76258493':self.starttime,
            ## End time
            'entry.1960199521':self.endtime,
            ## Disclaimer (checkbox)
            'entry.1234664796':'I read and agree to the disclaimer note.'
        }
        self.form = sheetfields
        return sheetfields

    def submitform(self):
        return self.gForm.submit()

## DO NOT ANYHOW CALL THIS FUNCTION
def submitform(data_dict):
    global url

    response = rq.post(url, data = data_dict)
    lg.functions.debug(f"response {response.status_code}")
    if (response.status_code == 200):
        return 1
    else:
        return 0
