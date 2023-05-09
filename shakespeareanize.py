#Shakespeare is chad frfr
def shakespeareanize(text):
    dictionary = {
        "hello": "hail",
        "hey": "hail",
        "'sup": "hail",
        "goodbye": "adieu",
        "friend": "companion",
        "enemy": "foe",
        "love": "affection",
        "hate": "loathe",
        "happy": "jocund",
        "sad": "melancholy",
        "beautiful": "fair",
        "ugly": "homely",
        "brave": "valiant",
        "coward": "craven",
        "king": "monarch",
        "queen": "sovereign lady",
        "prince": "nobleman",
        "princess": "maiden",
        "castle": "fortress",
        "sword": "blade",
        "horse": "steed",
        "ship": "vessel",
        "war": "battle",
        "peace": "tranquility",
        "book": "tome",
        "letter": "missive",
        "speech": "oration",        
        'you': 'thou',
        'your': 'thy',
        'yours': 'thine',
        'you\'re': 'thou art',
        'you\'ve': 'thou hast',
        'you\'ll': 'thou wilt',
        'y\'all': 'ye',
        'are': 'art',
        'am': 'be',
        'is': 'be',
        'was': 'wert',
        'were': 'wert',
        'have': 'hast',
        'has': 'hath',
        'had': 'hadst',
        'do': 'dost',
        'does': 'doth',
        'did': 'didst',
        'will': 'wilt',
        'shall': 'shalt',
        'should': 'shouldst',
        'may': 'mayst',
        'might': 'mightst',
        'must': 'must',
        'can': 'canst',
        'could': 'couldst',
        'would': 'wouldst'
    } #Big nerd moment
    
    words = text.split()
    shakespearean_text = []
    for word in words:
        if word.isupper():
            shakespearean_text.append(dictionary.get(word.lower(), word))
        elif word[-1] in [',', '.', '!', '?', ';', ':']:
            punctuation = word[-1]
            word = word[:-1]
            shakespearean_text.append(dictionary.get(word.lower(), word) + punctuation)
        else:
            shakespearean_text.append(dictionary.get(word.lower(), word))
    return ' '.join(shakespearean_text)


#Change it alllll up

sentence = input(str("Enter a sentence:\n"))

print(shakespeareanize(sentence)) #Ye
