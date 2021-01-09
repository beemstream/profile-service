import http from 'k6/http';
import { sleep, check  } from 'k6';

export default function () {
    const res = http.post('http://localhost:8000/login', JSON.stringify({
        identifier: 'foobar',
        password: 'WhatIsLife123'
    }), { headers: { 'Content-Type': 'application/json' } });

    check(res, {
        'is status 200': (r) => r.status === 200,
    });

    sleep(1);
}
