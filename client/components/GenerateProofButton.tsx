'use client'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Loader2 } from 'lucide-react'
import { useRouter } from 'next/navigation'
import { useState } from 'react'
import { toast } from './ui/use-toast'

export default function GenerateProofButton({
	imageId,
	argumentType
}: {
	imageId: string
	argumentType: string[]
}) {
	const router = useRouter()
	const [inputValues, setInputValues] = useState<(number | null)[]>([])
	const [isValid, setIsValid] = useState(false)
	const [isLoading, setIsLoading] = useState(false)

	const handleInputChange = (index: number, newValue: string) => {
		const newInputValues = [...inputValues]

		newInputValues[index] =
			newValue.match(/^\d*$/) && Number(newValue) > 0 ? Number(newValue) : null

		setInputValues(newInputValues)
		setIsValid(newInputValues.filter((v) => !!v).length === argumentType.length)
	}

	const handleSubmission = async () => {
		if (!isValid || !imageId || inputValues.length !== argumentType.length) return
		setIsLoading(true)

		const data = {
			image_cid: imageId,
			arguments: argumentType.map((a, i) => ({
				value: inputValues[i]?.toString(),
				arg_type: a
			}))
		}

		const res = await fetch(`http://localhost:3005/api/proofs`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(data)
		})

		if (!res.ok) {
			toast({ title: 'Failed to generate proof', variant: 'destructive' })
		} else {
			setTimeout(() => {
				setIsLoading(false)
				router.refresh()
			}, 1000)
		}
	}

	return (
		<>
			<Popover>
				<PopoverTrigger asChild>
					<Button variant="secondary">Generate Proof</Button>
				</PopoverTrigger>
				<PopoverContent className="w-80">
					<div className="grid gap-4">
						<div className="space-y-2">
							<h4 className="font-medium leading-none">Inputs</h4>
							<p className="text-sm text-muted-foreground">Set the inputs for proof generation.</p>
						</div>
						<div className="grid gap-2">
							{argumentType.map((a, i) => (
								<div className="grid grid-cols-3 items-center gap-4">
									<Label>
										Arg {i + 1} ({a.toLowerCase()})
									</Label>
									<Input
										value={inputValues[i] || ''}
										pattern="[0-9]+"
										className="col-span-2 h-8"
										placeholder={`Enter ${a.toLowerCase()} value`}
										onChange={(e) => handleInputChange(i, e.target.value)}
									/>
								</div>
							))}
						</div>
						<Button variant="default" disabled={!isValid || isLoading} onClick={handleSubmission}>
							{isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
							Generate
						</Button>
					</div>
				</PopoverContent>
			</Popover>
		</>
	)
}
