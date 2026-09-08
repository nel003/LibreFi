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

    const segmentsCount = 2 + (displayHours ? 1 : 0) + (displayDays ? 1 : 0) + (displayMonths ? 1 : 0);

    let adaptiveTextSize = "text-[29cqw]";
    if (segmentsCount === 3) adaptiveTextSize = "text-[19cqw]";
    if (segmentsCount === 4) adaptiveTextSize = "text-[14cqw]";
    if (segmentsCount === 5) adaptiveTextSize = "text-[11cqw]";

    return (
        <div className={`${className} ${adaptiveTextSize} flex items-center justify-center gap-[1.5cqw] w-full`}>
            <NumberFlowGroup>
                {displayMonths && (
                    <>
                        <div>
                            <NumberFlow className="text-primary" value={mo} format={{ minimumIntegerDigits: 2 }} />
                            <div className="-mt-[2cqw] text-[3cqw] text-foreground/50 text-center pt-[1cqw] tracking-normal w-full">
                                <span>MONTH</span>
                            </div>
                        </div>
                        <div className="text-foreground/10 -mt-[6cqw]">:</div>
                    </>
                )}
                {displayDays && (
                    <>
                        <div>
                            <NumberFlow className={displayMonths ? "text-foreground/90" : "text-primary/95"} value={d} format={{ minimumIntegerDigits: 2 }} />
                            <div className="-mt-[2cqw] text-[3cqw] text-foreground/50 text-center pt-[1cqw] tracking-normal w-full">
                                <span>DAY</span>
                            </div>
                        </div>
                        <div className="text-foreground/10 -mt-[6cqw]">:</div>
                    </>
                )}
                {displayHours && (
                    <>
                        <div>
                            <NumberFlow className={displayDays ? "text-foreground/90" : "text-primary/95"} value={h} format={{ minimumIntegerDigits: 2 }} />
                            <div className="-mt-[2cqw] text-[3cqw] text-foreground/50 text-center pt-[1cqw] tracking-normal w-full">
                                <span>HOUR</span>
                            </div>
                        </div>
                        <div className="text-foreground/10 -mt-[6cqw]">:</div>
                    </>
                )}
                {/* Minutes and seconds always display as the minimum format */}
                <div>
                    <NumberFlow className={displayHours ? "text-foreground/90" : "text-primary/95"} value={m} format={{ minimumIntegerDigits: 2 }} />
                    <div className="-mt-[2cqw] text-[3cqw] text-foreground/50 text-center pt-[1cqw] tracking-normal w-full">
                        <span>MIN</span>
                    </div>
                </div>
                <div className="text-foreground/10 -mt-[6cqw]">:</div>
                <div>
                    <NumberFlow className="text-foreground/70" value={s} format={{ minimumIntegerDigits: 2 }} />
                    <div className="-mt-[2cqw] text-[3cqw] text-foreground/50 text-center pt-[1cqw] tracking-normal w-full">
                        <span>SEC</span>
                    </div>
                </div>
            </NumberFlowGroup>
        </div>
    );
}

const Timer = ({ user }: { user: UserType | undefined }) => {
    const [sec, setSec] = useState(100);

    useEffect(() => {
        (() => {
            if (user) {
                if (user.paused) {
                    setSec(Math.max(0, user.expires_on - user.paused_on))
                    return;
                }
                setSec(Math.max(0, user.expires_on - user.now))
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
        <div className="@container w-full flex justify-center">
            <NumberTicker
                seconds={sec}
                className="text-foreground font-semibold tabular-nums tracking-tighter"
            />
        </div>
    );
};

export default Timer;