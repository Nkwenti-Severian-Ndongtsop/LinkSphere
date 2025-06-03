import { useMemo } from 'react';
import { Check, X } from 'lucide-react';

interface PasswordRequirement {
    regex: RegExp;
    text: string;
}

const PASSWORD_REQUIREMENTS: PasswordRequirement[] = [
    { regex: /.{8,}/, text: 'At least 8 characters long' },
    { regex: /[A-Z]/, text: 'Contains uppercase letter' },
    { regex: /[a-z]/, text: 'Contains lowercase letter' },
    { regex: /[0-9]/, text: 'Contains number' },
    { regex: /[^A-Za-z0-9]/, text: 'Contains special character' },
];

interface PasswordStrengthIndicatorProps {
    password: string;
    className?: string;
}

export default function PasswordStrengthIndicator({ password, className = '' }: PasswordStrengthIndicatorProps) {
    const requirements = useMemo(() => {
        return PASSWORD_REQUIREMENTS.map(requirement => ({
            ...requirement,
            isValid: requirement.regex.test(password),
        }));
    }, [password]);

    const strength = useMemo(() => {
        const validRequirements = requirements.filter(req => req.isValid).length;
        if (validRequirements === 0) return 0;
        return (validRequirements / PASSWORD_REQUIREMENTS.length) * 100;
    }, [requirements]);

    const strengthColor = useMemo(() => {
        if (strength < 40) return 'bg-red-500';
        if (strength < 80) return 'bg-yellow-500';
        return 'bg-green-500';
    }, [strength]);

    return (
        <div className={`space-y-4 ${className}`}>
            <div className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
                <div
                    className={`h-full transition-all duration-300 ${strengthColor}`}
                    style={{ width: `${strength}%` }}
                />
            </div>

            <div className="space-y-2">
                {requirements.map((requirement, index) => (
                    <div
                        key={index}
                        className={`flex items-center space-x-2 text-sm ${
                            requirement.isValid ? 'text-green-600 dark:text-green-400' : 'text-gray-600 dark:text-gray-400'
                        }`}
                    >
                        {requirement.isValid ? (
                            <Check className="h-4 w-4" />
                        ) : (
                            <X className="h-4 w-4" />
                        )}
                        <span>{requirement.text}</span>
                    </div>
                ))}
            </div>
        </div>
    );
} 