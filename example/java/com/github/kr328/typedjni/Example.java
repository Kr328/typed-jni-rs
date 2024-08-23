package com.github.kr328.typedjni;

public class Example {
    public static native void nativeFunction(int value1, float value2, String value3);

    public static void run() {
        nativeFunction(123, 4.4f, "114514");
    }

    static {
        System.loadLibrary("example");
    }
}