"use client";

import { useEffect, useState } from "react";
import NumberFlow, { NumberFlowGroup } from "@number-flow/react";
import type { UserType } from "../types/user";

type NumberTickerProps = {
    seconds: number;
    className?: string;
    showMonths?: boolean;
    showDays?: boolean;
    showHours?: boolean;
};

function NumberTicker({
    seconds,
    className,
    showMonths = true,
    showDays = true,
    showHours = true,
}: NumberTickerProps) {
    const SECONDS_IN_MINUTE = 60;
    const SECONDS_IN_HOUR = 3600;
    const SECONDS_IN_DAY = 86400;
    const SECONDS_IN_MONTH = 2592000;

    const mo = Math.floor(seconds / SECONDS_IN_MONTH);
    const d = Math.floor((seconds % SECONDS_IN_MONTH) / SECONDS_IN_DAY);
    const h = Math.floor((seconds % SECONDS_IN_DAY) / SECONDS_IN_HOUR);
    const m = Math.floor((seconds % SECONDS_IN_HOUR) / SECONDS_IN_MINUTE);
    const s = seconds % SECONDS_IN_MINUTE;

    // Cascading logic: Only show a segment if it's > 0 OR if a larger unit exists
    const displayMonths = showMonths && mo > 0;
    const displayDays = showDays && (displayMonths || d > 0);
    const displayHours = showHours && (displayDays || h > 0);

    return (
        <div className={`${className} flex items-center gap-2`}>
            <NumberFlowGroup>
                {displayMonths && (
                    <>
                        <div>
                            <NumberFlow className="text-primary" value={mo} format={{ minimumIntegerDigits: 2 }} />
                            <div className="-mt-2 text-xs text-foreground/50 text-center pt-2 tracking-normal w-full">
                                <span>MONTH</span>
                            </div>
                        </div>
                        <div className="text-foreground/10 -mt-6 lg:-mt-7">:</div>
                    </>
                )}
                {displayDays && (
                    <>
                        <div>
                            <NumberFlow className={displayMonths ? "text-foreground" : "text-primary/95"} value={d} format={{ minimumIntegerDigits: 2 }} />
                            <div className="-mt-2 text-xs text-foreground/50 text-center pt-2 tracking-normal w-full">
                                <span>DAY</span>
                            </div>
                        </div>
                        <div className="text-foreground/10 -mt-6 lg:-mt-7">:</div>
                    </>
                )}
                {displayHours && (
                    <>
                        <div>
                            <NumberFlow className={displayDays ? "text-foreground" : "text-primary/95"} value={h} format={{ minimumIntegerDigits: 2 }} />
                            <div className="-mt-2 text-xs text-foreground/50 text-center pt-2 tracking-normal w-full">
                                <span>HOUR</span>
                            </div>
                        </div>
                        <div className="text-foreground/10 -mt-6 lg:-mt-7">:</div>
                    </>
                )}
                {/* Minutes and seconds always display as the minimum format */}
                <div>
                    <NumberFlow className={displayHours ? "text-foreground" : "text-primary/95"} value={m} format={{ minimumIntegerDigits: 2 }} />
                    <div className="-mt-2 text-xs text-foreground/50 text-center pt-2 tracking-normal w-full">
                        <span>MIN</span>
                    </div>
                </div>
                <div className="text-foreground/10 -mt-6 lg:-mt-7">:</div>
                <div>
                    <NumberFlow value={s} format={{ minimumIntegerDigits: 2 }} />
                    <div className="-mt-2 text-xs text-foreground/50 text-center pt-2 tracking-normal w-full">
                        <span>SEC</span>
                    </div>
                </div>
            </NumberFlowGroup>
        </div>
    );
}

const Timer = ({ user }: { user: UserType | undefined }) => {
    const [sec, setSec] = useState(0);

    useEffect(() => {
        (() => {
            if (user) {
                if (user.paused) {
                    setSec(user.expires_on - user.paused_on)
                    return;
                }
                setSec(user?.expires_on - user?.now)
            }
        })()
    }, [user]);

    useEffect(() => {
        if (!user || user.paused) return;

        const timer = setInterval(() => {
            setSec((prev) => (prev > 0 ? prev - 1 : 0));
        }, 1000);

        return () => clearInterval(timer);
    }, [user]);

    return (
        <div>
            <NumberTicker
                seconds={sec}
                className="text-foreground font-semibold tabular-nums tracking-tighter text-5xl md:text-6xl lg:text-8xl"
            />
        </div>
    );
};

export default Timer;