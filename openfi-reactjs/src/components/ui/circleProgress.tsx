"use client";

import { cn } from "#lib/utils";

interface CircularProgressProps {
    value: number;
    renderLabel?: (progress: number) => number | string;
    size?: number;
    strokeWidth?: number;
    circleStrokeWidth?: number;
    progressStrokeWidth?: number;
    shape?: "square" | "round";
    className?: string;
    progressClassName?: string;
    labelClassName?: string;
    showLabel?: boolean;
}

const CircularProgress = ({
    value,
    renderLabel,
    className,
    progressClassName,
    labelClassName,
    showLabel,
    shape = "round",
    size = 100,
    strokeWidth,
    circleStrokeWidth = 10,
    progressStrokeWidth = 10,
}: CircularProgressProps) => {
    const clampedValue = Math.max(0, Math.min(100, value));
    const radius = size / 2 - 10;
    const circumference = Math.ceil(3.14 * radius * 2);
    const percentage = circumference * ((100 - clampedValue) / 100);

    const viewBox = `-${size * 0.125} -${size * 0.125} ${size * 1.25} ${size * 1.25
        }`;

    return (
        <div className="relative">
            <svg
                className="relative"
                height={size}
                style={{ transform: "rotate(-90deg)" }}
                version="1.1"
                viewBox={viewBox}
                width={size}
                xmlns="http://www.w3.org/2000/svg"
            >
                {/* Base Circle */}
                <circle
                    className={cn("stroke-primary/25", className)}
                    cx={size / 2}
                    cy={size / 2}
                    fill="transparent"
                    r={radius}
                    strokeDasharray={circumference}
                    strokeDashoffset="0"
                    strokeWidth={strokeWidth ?? circleStrokeWidth}
                />

                {/* Progress */}
                <circle
                    className={cn("stroke-primary", progressClassName)}
                    cx={size / 2}
                    cy={size / 2}
                    fill="transparent"
                    r={radius}
                    strokeDasharray={circumference}
                    strokeDashoffset={percentage}
                    strokeLinecap={shape}
                    strokeWidth={strokeWidth ?? progressStrokeWidth}
                    style={{ transition: "stroke-dashoffset 220ms ease-out, stroke 220ms ease-out" }}
                />
            </svg>
            {showLabel && (
                <div
                    className={cn(
                        "absolute inset-0 flex items-center justify-center text-md",
                        labelClassName
                    )}
                >
                    {renderLabel ? renderLabel(value) : value}
                </div>
            )}
        </div>
    );
};

export default CircularProgress;