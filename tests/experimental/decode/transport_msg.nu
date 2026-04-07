#!/usr/bin/env nuze -X0

use std/assert

let msg = "pYm+1ngxAN8BIQjHAQEGQH8CARN0Y3AvMTI3LjAuMC4xOjQyNjg3fjMgDho8tAHsAdsBEAQfDAWsAY8BMugBnwGxASaVAUpIlgEVKzsYJSLpAcgBdiGRATmSAZMBP54BAZkBSREdNCMJjgEk1AGYAaEBVycZR6YBMa4Bci4XG1sDnQEoN7sBOqoBTClCLEOrAQ8GRC8TFsEBmwEtjQGcASqkAQAKqQHjAUWaAaIBCBwUcNoBvgGlATijAacBPgKoAbABwAE2lAELMEYHPQ0eQRI1lwE="

let expected = {
    type: frame
    reliability: "1"
    sn: 253075209
    messages: [
        [type reliability id ext_body linkstate];
        [
            oam
            "1"
            1
            zbuf
            [
                [psid sn zid whatami locators links link_weights];
                [
                    64
                    127
                    null
                    peer
                    ["tcp/127.0.0.1:42687"]
                    [51 32 14 26 60 180 236 219 16 4 31 12 5 172 143 50 232 159 177 38 149 74 72 150 21 43 59 24 37 34 233 200 118 33 145 57 146 147 63 158 1 153 73 17 29 52 35 9 142 36 212 152 161 87 39 25 71 166 49 174 114 46 23 27 91 3 157 40 55 187 58 170 76 41 66 44 67 171 15 6 68 47 19 22 193 155 45 141 156 42 164 0 10 169 227 69 154 162 8 28 20 112 218 190 165 56 163 167 62 2 168 176 192 54 148 11 48 70 7 61 13 30 65 18 53 151]
                    null
                ]
            ]
        ]
    ]
}

assert equal ($msg | decode base64 | zenoh decode transport-msg) $expected
