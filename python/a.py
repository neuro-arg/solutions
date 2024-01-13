#!/usr/bin/env python3
import io
import struct

from base64 import b64decode
# pip install pycryptodomex
from Cryptodome.Cipher import AES
from typing import List, Union

POW3 = [3 ** i for i in range(11)]

def encode(x: Union[str, bytes]) -> bytes:
    if isinstance(x, bytes): return x
    return x.encode('utf-8')

def decode(x: Union[str, bytes]) -> str:
    if isinstance(x, str): return x
    return x.decode('utf-8')

def fib(n: int) -> List[int]:
    ret = [0, 1]
    while len(ret) < n: ret.append(ret[-1] + ret[-2])
    return ret[:n]

def shift_characters(message: str, n: int = 1) -> str:
    if not message.replace(' ', ''): raise ValueError()
    return ''.join(chr(max(ord(' '), ord(char) - n)) for char in message)

def malbolge(code: Union[str, bytes], input: Union[str, bytes] = b'') -> bytes:
    code = b''.join(encode(code).split())
    if not all(map(lambda x: (x[0]+x[1])%94 in b"\4\5\x17'(>DQ", enumerate(code))): raise ValueError('invalid opcode at load')
    mem = [*code, *range(POW3[10] - len(code))]
    crazy = lambda a, b: sum(p*b'\1\0\0\1\0\2\2\2\1'[((a//p)%3)*3+(b//p)%3] for p in POW3[:10])
    for i in range(len(code),POW3[10]): mem[i] = crazy(*mem[-2:])
    inp = io.BytesIO(encode(input))
    ret = io.BytesIO()
    a = c = d = 0
    while True:
        if mem[c] not in range(33, 127): break
        v = (mem[c]+c) % 94
        if v == 4: c = mem[d]
        elif v == 5: ret.write(bytes([a % 256]))
        elif v == 23: a, = inp.read(1)
        elif v == 39: mem[d] = a = mem[d] % 3 * POW3[9] + mem[d] // 3
        elif v == 40: d = mem[d]
        elif v == 62: mem[d] = a = crazy(mem[d], a)
        elif v == 81: break
        if mem[c] in range(33, 127):
            mem[c] = b'5z]&gqtyfr$(we4{WP)H-Zn,[%\\3dL+Q;>U!pJS72FhOA1CB6v^=I_0/8|jsb9m<.TVac`uY*MK\'X~xDl}REokN:#?G"i@'[mem[c] - 33]
        c = (c + 1) % POW3[10]
        d = (d + 1) % POW3[10]
    return ret.getvalue()

# hex to bytes
def h2b(x: str) -> bytes:
    return bytes(int(x[i:i+2], 16) for i in range(0, len(x), 2))

def xor(key: bytes, x: bytes) -> bytes:
    return bytes(x[i] ^ key[i % len(key)] for i in range(len(x)))

def enc1(x1: int) -> str:
    x = int(f'2{x1}91')
    x *= 5
    x = int(f'{x}6'[::-1].replace('2', '3'))
    x *= 9
    x = int(f'17{x}24')
    return replace_all(str(x), str(x1), 'abcdefghijklmnopqrstuvwxyz'[:len(str(x1))])

def dec1(y: str) -> int:
    x = int(replace_all(y, 'abcdef', '572943'))
    assert x % 100 == 24 and str(x).startswith('17')
    x = int(str(x // 100)[2:])
    assert x % 9 == 0
    x //= 9
    x = int(str(x).replace('3', '2')[::-1])
    assert x % 10 == 6
    x //= 10
    assert x % 5 == 0
    x //= 5
    assert x % 100 == 91 and str(x).startswith('2')
    return int(str(x // 100)[1:])

def replace_all(s: str, a: str, b: str) -> str:
    m = {k: v for k, v in zip(a, b)}
    return ''.join(map(lambda x: m.get(x, x), s))

def unpad(s: bytes) -> bytes: return s[:-int(s[-1])]

def decrypt(key: bytes, data: bytes) -> bytes:
    cipher = AES.new(key, AES.MODE_ECB)
    return unpad(cipher.decrypt(data))

NUMBERS_KEY = encode(enc1(572943))

def numbers() -> str:
    # https://www.youtube.com/watch?v=wc-QCoMm4J8
    data = b64decode('BvmSc+u5NXEjBP3z2gz57G3T8p0sc25U8Bv9yePOa3J6xGRhYcgUL6mnE2eMuFVvAHS5X1QWToqZtG13shERSSM928pZoOpWVQSR0eCZoJU+2FtB8Sd91pArvr5csHjsO2VjSwSgajrWbQLNuisad5cFvGYC/OLopisMBRHmUB+Dl/kqZleP7bF5WgpSxKRDKLjjqWlR95y9a/96vY3vc+gdKb/V4Sd8AIeyIryv53HKFRdB3IL7WOVSr8RSgFbb5Z3KTezpDdA9VvDIGf3pK5TBXpNdJs1g1pnV5C6QvVw7I/ZFhDFqoogGlsbBY0nSihzgvhzM86qgh6OqkYsKwrsdrvwuHTwz2MfzKRBPGGpKYwNmOpgf2dMgT+9GHGWfg/wLvynRqmzyhfyUSLR8tBFBDyOmwKF34Lz7PrVguu47RVawtcyZFRfHvW1rSVfPILi+1JXC5slE2PK84VdutLg4/NGOxC+s1wkYbBRC6ty970raEXKGMZeeLEsLrJhRmh1jtRltCI1zgpoxVhpgUZ/n832SKyClICQpLFKHG8yZ4wcMUA2yAN7b9sXh3nGQxkN9M8g1AuBZez5OsbgC5S9ypO8UMOrhr4f8pwVro15idoDYlg2nO72dpcWUkXBLcDq6h81Y7yaKN3IbrMEqdU9eTYwGCu824OitkxEMuCyQh9vB+rbu6svT9xyUqDLGllM4aF7v681y96CaQ+S2Gg9gmjAHfcR8AfIqGI+qrVZYj9ibcsO/bjPqK/Qni1Ti/4QuznYBQ67LZoU1mp1TOOgpN4LN9OQra65+CSpe/zC1+3JKEVPH4ml4SVoNzunUVoZccFTP86pfA2vwRtd8btPj3cyVRLKvcmj4qxgu1q74mjcgvYsrye4kCUE3MsjL6RYJI6vED+xWYvFhiJGo8+GqUci7Jge//iIaTlAnVtyxzIJjFgrtLyapd+/AM8QtZBYNDC+zKGu6haBhxTlOUyUiIA6SOVWVgiGzz887ieOoiUf8qopHJO9Mptrb16Nh1aPFE7XqDTCZUXn2MU/N7/OFMuRt0DO8/x1A0f9pXuo01uYjDBXQTCDp8Xu6YhvIMgFdC5mYpFJSTuUYSTILhKrm22P0q0kA0eB0O9Rvpi1H0MP2MpdwZ6eB7pDM75MpbC/KOSuv6wb9mhCj9PPCO32+AGltL6m7l+ciFAjrscn8Ych85d970dJc4OSys3LCfZ9OupDFhCqRUAVku/rUdRbup89e0Kf0bD0QSs3Ths5ueQOYxB2FRDA7Pp7IU2EcpM4c9xhGJnaPCE11nihfdEZk9N6T/tOUhREqRgm2kiU54NRZGvi9jCOlQjDYAwDoMMruBND5AHJ3gIHeHY4nYJ9UoR85ZKjHo6F3LsEwjF0=')
    print(NUMBERS_KEY, data)
    return decode(decrypt(NUMBERS_KEY, data))

def study1() -> str:
    # https://www.youtube.com/watch?v=zMlH7RH6psw&t=8m39s
    return '''
ANXIETY
A supernatural force that stops us from being human
'''.strip()

def study2() -> str:
    # https://www.youtube.com/watch?v=zMlH7RH6psw&t=10m21s
    b64 = 'DQ5z14ighWwlag7y+cWFQg=='
    data = b64decode(b64)
    # +44 7537 130663
    phone = decode(decrypt(NUMBERS_KEY, data))
    return '''
const lib = require('messagemedia-messages-sdk');

var controller = lib.MessagesController;

let body = new lib.SendMessagesRequest();

body.messages = [];

body.messages[0] = new lib.Message(); 	 

body.messages[0].content = 'save me...';
body.messages[0].destinationNumber = 'DQ5z14ighWwlag7y+cWFQg==';

controller.sendMessages(body, function(error, response, context) {
  if (error) {
    console.log(error);
  } else {
    console.log(response);
  }
});
'''.strip().replace(b64, phone)

def study3() -> str:
    # https://www.youtube.com/watch?v=zMlH7RH6psw&t=31m10s
    code = '''D'`;M^"~~;{X2V0/eu,s*/(LJ%*j"XEVUAAb
Qav<)(xqYonsrqj0hmlkjc)a`_^$#a`BX|\\U=
Sw:VUTMqQ32NMLEDhHGF?'C<`#?8\\<;:z8
1UTu-,+Op.-&+$)"F&f$#zy~w=uzyrwp654r
qpRQ.ONdiba'_^]ba`Y}]V[ZSwWP8NMLpJO
NGFKJIHAeEDCBA:^>7}5Y98x6543,P0po'
&%I#('&%${A!awv{zs98YXtmlk1onmfN+L
ha`_%FEaZ_X]Vz=SXQVUNrR43ImML.Dh
HA@EDCB;_"!=<5Yzy705432+0/(L,l$H('&fe
B"b~}v<z\\xwp6tsrkpong-NMiha'ed]ba`_X|
V[ZSwQVOsS54PIm0FEDIHGF?>bB$@987[
;:z2765.RQPO/.-m%*#G'&%|d"y?}_{t:xwYu
tm3qpihg-kMLha'eGcba`Y}]\\U=Sw:VUTMq
Q32NMLEDhH*@ED=<`@?>=<|{92V0/43,+
ONon&%I)(h&%$#z@a}v{zyr8YXnml2pon
mlkjiba'&^F\\"CB^W\\UyYXWVU7Mq43ImG
FEJIBfeED&B;_?!=654Xyx6/St,+O/o-&+$#G
'~%|{"y?w|u;yxqYon4!'''
    return decode(malbolge(code))

def numbers2_1() -> str:
    # neuro sequence
    aes_key = b'10100101101001011101001010101001'
    data = b64decode('x/26YzndR5VhxhR5tGvNJKOfHX2DL3qYXgyJvVQ5EJi6jH/wmI21ftjY+i5GstZRo0gHxRmEmS4iavJfRYrvtdrUXBSpZPiRi25e9HMV0FimPaDxMrVD8P4/VgYFWh01X92ftE5IlyNDasz4LCkWPiULaPGg+Et2AMzGqtLK7C0T/04wDgbqhYkcPBemStjxJcyfKjgXcMa3LJ6IQPaMakmgm2R3G38jIB/YNBZt3p4Ixc6VdvThxszTH1CxTFtKkwua2QsZFQaGmoOBx9n77TlpIFgxNz6PQyI5gQ0bcUI6szKQgAezC5tFT9lHhQVuau0xABiWMXX+m39RE45a0aE1/8G45FuetjoVEsTbN7rPdmIGqO2BsPPqzwqT6ZItCqmhYLZnN6peOIBJEK+okS77a1v4KDyecoKjt8Tx8zBB9oH52o9S5gwarJlteDNVM3UfFGQwrCzyDnmkMEqfio2f/Upfs69BDIhitivUtfPPH3o7swOcEyD3YjCvuVfuSh+lpfBT7TicxVCGKjrNMfI8s4GuPq6VH+MnDHbLy/LZTbwbWJbfAFIi/qprXGAGgh+anQynb+J4458YQVjquLgsBdJo4aUWV4ccU9XSBnnrhIQIfTkdMyXPKNA5lnSdkGeVoIZhMUTO953xOMwtRovNw4vV12FVq+ltlqE/AkQfA14C12ew7C66K+QPRkPw/lChRrzL8N3qaBDV5Ak4T+NM34qGgR+hTvQs49xH34fc2S9axi6Pcav08RdKvTtQoXkUHGmp3ueWPPrFPXqbcreXHm4p8HiTJycwlWYrfbVcU5dxgoCc0Sb9N8tF8QkqC7OxkzM6sjFFJoY4mQSgUEdJmF9T2G/U/w1tcgBS3Kr2v4NqoKX/WeN+O+gQTd0AFzK3HvLpc4G5384rwTnqfJz521mrBFZvhrkwfJ+TjjFt6Dag/LtN2e3woDYxh39XB3iKE3bxBCmZhZ2xAvNo08+pf6zeMgPHnnOzGjiIiFKGojpn+tfrr9ZhDY5Wzzllno8U2Wr8aVgqowXaziokq5vCK2EGlG26xTGTfNN2nRqlPuhJqDPSQU2tuJd7/4h6DXYmMdI25cqvWglt0zT9VEwZwLwMVG/C0KrHV70fBz6Q2rAjdG99HpP94M+LNy1MPFCmdDLVB3ecDVQink7igGt3+KJRlB0hqdRVHmsitBVOwjmfENKyew0q8+Q+9BE+n6t1ebPW2K4a+wZemFaLuZXSpoachDj3KC7JSMKwaVtXnLs8h6+UxA+M2ddcJkVsL6lF79tT4Q0IrtPUqj8L6eZjRR5XPALMtIPkFOtyakwtKYy9NP3I1wr6wnddRUTemyGyHmJAPaQST6T7nSPzXRSuZ8sPIoDEuaJIyl19JDyIIE+4HasXmD+dVBBiYG1qQqK4a5B2tne0T/jHCbr8MFVAPFJP8mnNM8s83TAjQh1KLeYn4T4tvPy/Be3b2Nn0cYSvpF8xGHXTLQqmGvESgN6o7eXBzt9ACBmJQYh1rTo/kMZ4sMhtZ39yYRSkUK8h6H20RX3LJxt5u7195siA3gddJeUSdVmiBVQg4OTR1r7PSbfPo5YQh0G73pK9eVf6aQQ02Okgi8Y6pr0gru/PZgpDXJtaaL0W9mjSwdSYwtZjLAIB0jxynd+O8VM7YhNCshBYh/Pz2btw51uqcpPCWA==')
    return decode(decrypt(aes_key, data))

def numbers2_2() -> str:
    data1 = numbers2_1()
    b64 = data1.split()[-1]
    data = b64decode(b64)
    fibs = ''.join(map(str, fib(30)))
    # from lyrics
    for nums in [[201, 1321, 5831], [723, 743, 4913], [879, 875, 2145], [716, 906, 4156]]:
        schizo = sorted(''.join(map(str, nums)))
        found = False
        for i in range(len(fibs) - len(schizo)):
            if sorted(fibs[i:i+len(schizo)]) == schizo:
                fibs = fibs[:i] + fibs[i+len(schizo):]
                found = True
                break
        if not found:
            raise ValueError('Not found!')
    aes_key = encode(fibs[:16])
    # print('a', aes_key)
    return data1.replace(b64, decode(decrypt(aes_key, data)))

def numbers2() -> str:
    data1 = numbers2_2()
    code = data1.split()[-1]
    return data1.replace(code, decode(malbolge(code)))

def soundcloud() -> str:
    data = b64decode('Mi9+FxIon9xfLnH5mJVisFF+9L7fTsttB5HjIJMH81O7Ifug4oGDRrTFyMdHSYrHXnAo5BK5UJT0dI9AnudS29L0+Qt6POsWJ74Un2O7AdtpqaAdWdBmyBbBKyLN+fqnr5CfEXMeHSAWGm8BgPW1TsODm6Kbk6vMIIG+B0FeHYSXZ0tRYyXTJgE/EmWDRdMl0fFCrKRgCeTvQ24HI3V8Z2iRgo1ZDHsANWBqMdrhVqZJvjA6moMpqKMTqmgK/QYL1KNGkxTpw8LcSL3R/QW62E4UHA9w9PP+N+48lVnvRttGBNbjAv4vmS8SJ/ckK0WB4OSRdbYrOIB619VFBx+NjRnaQUJCJyomfsTQEVsj4F/wmLF8C4jOy1/TALfL+qGA3cnnHfD5yaV9pdAM5zRFvxpdfuCQioAW31RWXoHfrPLzyN6s4QA83zus/8HlT1iAcIyG5brYlRC76CBZN6HpvEh2X5535abSg2pkVrmMu6VMpsad1+4BiIAGFSdw8Mt1yTeWTZ1TX6gUn0/amcxGwg==')
    image = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAABNgAAABkAQMAAABuPx8iAAAAA1BMVEX///+nxBvIAAAAJUlEQVR4Xu3AMQEAAADCIPunNsYeWAQAAAAAAAAAAAAAAAAAAOA88AABMGQBNwAAAABJRU5ErkJggg=='
    key_data = image.split('Y')[-2].split('X')[-1]
    # "odd one out"
    aes_key = encode(key_data.replace('3', ''))
    return decode(decrypt(aes_key, data))

def filtered() -> str:
    title = '\u0354\u0337\u0370\u0334\u0381\u036d\u0331\u037e\u034c\u036a\u034a\u0349\u0366\u0365\u0340\u0386\u0351\u0385\u0373\u0382\u0381\u033f\u037f\u033d\u0349\u033b\u036b\u037b\u0338\u0355\u0378\u0366\u0376\u0333\u038a\u0350\u034f\u034e\u035f\u038b\u0339\u035c\u0382\u0387\u0386\u037f\u0384\u037d\u0343\u0381\u037a\u037f\u0378\u033e\u035f\u035e\u037a\u0379\u0378\u0371\u0337\u0336\u0374\u0356\u0372\u036b\u0370\u0369\u0368\u038c\u036c\u034e\u034d\u0369\u0368\u0367\u0360\u035f\u0383\u035d\u0362\u035b\u035a\u0359\u035e\u035d\u0356\u037a\u035a\u0359\u0358\u0351\u0350\u0374\u0337\u0353\u0352'
    data = b64decode('L8gnk0GgYm9cCAA5t7wSIcwc669T+2TY/KlK4ATmQsnVrSY3PWrXAwUPcJiN1AugpkVwkDQARQydWkbLvs+4V0I08mSQKRsDinKZchmMlTJY8KCCS4ZDof1BxuCB7Uab6rAitGRYl+KgXqvROEbCWfb80nsDNaqo6wavnAVX5ld3nD6Ykl0vKIUUVNxuE42xDiMYuENg+tFLwsKcUzuw2KNZ+st46FBkZBniKP5jVQrqZzqAzgvcpHR63yMOZPkWMrVBHBwCRS31GRVx5qpzoB+0dkP0vX+YugYKIe9HvkEFJ440PCpMSd3ITK5Zmq/YJfAg5YyNpmRod3b2MVOfhX35lkjg4l+4bidTo4H4d8sTiNz7YD74a5tWuzCj6BXax7ErPueqA7uRCcjaNXnGGrDLaFsEQkpFKWRmWm3hltAF5FiTqfB//7zj10iWCBDt3jJ1uNhrFLG7SX8kpvFyuw==')
    original_code = shift_characters(title, 784)
    # the end was generated using an online malbolge generator
    code = original_code + ';:9]7<54981U543s+O)o-&+*#G4'
    assert malbolge(code) == b'hello world!'
    aes_key = encode(code[-16:])
    return decode(decrypt(aes_key, data))

def meaning_of_life():
    data = b64decode('jeISomyoFEJcqVt9NRBYsaD8OLh2Wx1qU4TotoFNDeKPwcZQynZBJA7pRGYzk12HbPXZnAHlt+nTa/AhJQ/bSuEOSH6ho5UOCCn5y4/bXlVFmtU+8NPgm8r4RC1p9dWwtzXIqi5FkLu3ur+0KNRR+AyPrwnX5+QaNtgbHAvwDJ6YwG+leyYtbwnsh2VHh/MRjhIXJiWIpRrFudLXi9eqb8wr+n49QbjlZaKD+iC9DQbcgikAfnBhJhFYRnHarfVZonyWMp9VfTZOwIBhWacHUHQQMpdshoMrURRIbO49Wvo6aUhX6Y2GAazFlodmRMdwOQ2mpjYH6owgY80z7mX50k60tB3nosrVh5Sc7DE+fgW4Nlt+hShVgsPz3g69XteejUle//VNFEK9kk3ds5f9cg==')
    binary = '01100011 00110111 00110101 01100101 00110000 01100110 01100010 00110000 00110101 01100101 01100101 01100011 00111000 00110111 00110111 01100110 01100011 00111000 00110101 00110010 00110010 01100101 00110101 00110101 00110000 01100100 01100110 00110101 00110101 01100100 01100101 01100100 00110101 01100010 00110101 00110000 00110101 00110000 00111000 01100110 01100010 01100010 01100101 00111000 00111000 01100010 01100010 00110111 01100100 00111000 00110010 01100101 00110010 00111000 01100100 00111000 01100110 00110010 01100100 01100110 00110000 01100101 00110010 01100010'
    # c75e0fb05eec877fc8522e550df55ded5b50508fbbe88bb7d82e28d8f2df0e2b
    # 64 hex chars
    # ['0', '2', '5', '7', '8', 'b', 'c', 'd', 'e', 'f']
    # 0   2     5   7 8     b c d e f                       - present
    #   1   3 4   6     9 a                                 - missing
    #     2 3 4 5   7   9                                   - 572943
    # 0     3 4       8     b   d                           - missing in hex(num)
    binary = ''.join(map(lambda x: chr(int(x, 2)), binary.split()))
    # 18 dec digits
    # 15 hex digits
    # 20 oct digits
    # 60 bin digits
    num = 692048501258949201
    #idk = int(replace_all(hex(num)[2:].zfill(16), 'abcdef', '572943'), 16)
    #print(idk)
    # 14+19=33 chars, 13 groups
    # 2 2 2 2 3 3
    # 4 3 2 2 4 1 3
    #     69 20 48 50 125 894 9201 99a a6 71 fce1 9 251
    # charset: bcdefghijqrstvy, 15 different chars
    # bb ce td ht eft ggd / sgfi dqj ie br vtye b sbs
    eh = 'bb ce td ht eft ggd sgfi dqj ie br vtye b sbs'
    #w = h2b('2235D48D56D774C7694A952BFDE52C2C')
    #s = h2b('1124C37C45C663B65839841AECD41B1B')
    # w = h2b('1124C37C45C663B6583F9841AEC41B1B')
    # bcdefghijqrstvy
    # bbcetdhteftggdsgfidqjiebrvtyebsbs
    # aabdscgsdesffcrfehcpihdaqusxdarar
    # aabdscgsdesffcrfehcpihdaqusxdarar
    # eh = ''.join(chr(ord('z') - ord(x) + ord('a')) if x != ' ' else x for x in eh)
    #print(binary, num, eh)
    #print(decrypt(b'U543s+O)o-&+*#G4', data))
    #print(enc1(num))
    return
    for i in range(1, 1000):
        eh = shift_characters(eh)
        print(i, eh)

print(enc1(906))

ALL = [
    numbers(),
    study1(),
    study2(),
    study3(),
    numbers2(),
    soundcloud(),
    filtered(),
    # public static void = just a hint, hopefully
    # hello world! = [Filtered] hint?
]

print('\n==========\n'.join(ALL))

#print(meaning_of_life())
