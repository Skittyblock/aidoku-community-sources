import { JSON } from "assemblyscript-json";
import { RegExp } from "assemblyscript-regex";
import LZString from "lz-string";

export class Decoder {
  // private pattern = /^.*\}\(\'(.*)\',(\d*),(\d*),\'([\w|\+|\/|=]*)\'.*$/g;

  private pattern = new RegExp("^.*}('(.*)',(d*),(d*),'([w|+|/|=]*)'.*$", "g");

  private func: string;
  private a: number;
  private c: number;
  private data: string[];

  constructor(document: string) {
    let match = this.pattern.exec(document);

    if (match != null) {
      this.func = match.matches[1];
      this.a = parseInt(match.matches[2]);
      this.c = parseInt(match.matches[3]);
      this.data =
        LZString.decompressFromBase64(match.matches[4])?.split("|") ?? [];
    } else {
      throw new Error("the document not good");
    }
  }

  e(c: number): string {
    let prefix: string;
    if (c < this.a) prefix = "";
    else prefix = this.e(Math.floor(c / this.a));

    let suffix = [
      this.tr(c % this.a, 36),
      String.fromCharCode((c % this.a) + 29),
    ][c % this.a > 35 ? 1 : 0];

    return prefix + suffix;
  }

  tr(value: number, num: number): string {
    let tmp = this.itr(value, num);
    if (tmp === "") return "0";
    else return tmp;
  }

  itr(value: number, num: number): string {
    let d = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    if (value <= 0) return "";
    else return this.itr(Math.floor(value / num), num) + d.charAt(value % num);
  }

  decode(): JSON.Obj {
    this.c = this.c - 1;
    let d = new Map<string, string>();
    while (this.c > -1) {
      d.set(
        this.e(this.c),
        [this.data[this.c], this.e(this.c)][this.data[this.c] == "" ? 1 : 0]
      );
      this.c = this.c - 1;
    }

    //   let pattern2 = /(\b\w+\b)/g;
    //   let pattern2 = new RegExp("(\b\w+\b)", "g")
    //   let pieces = this.func.split(pattern2)
    let pieces = this.func.split("");

    let arr = pieces.map((x) => (d.has(x) ? d.get(x) : x));
    let js = arr.join("");

    let pattern3 = new RegExp("^.*(({.*})).*$", "g");
    let match = pattern3.exec(js);
    if (match == null) throw new Error("something's wrong");
    return <JSON.Obj>JSON.parse(match.matches[1]);
  }
}
