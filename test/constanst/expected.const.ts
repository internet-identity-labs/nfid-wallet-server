import { getFile } from "../util/call.util";

export const Expected = {
    ACCOUNT: (x: string = "null", y: string) => getFile("/account.txt", x, y),
    ERROR: (x: string, y: string) => getFile("/error.txt", x, y),
    BOOL: (x: string, y: string) => `(record { data = opt ${x}; error = null; status_code = ${y} : nat16 })`
}

