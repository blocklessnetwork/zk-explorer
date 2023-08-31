'use client'

import { useRouter } from 'next/navigation'
import { Button } from './ui/button'
import { Input } from './ui/input'
import { useEffect, useState } from 'react'
import { CID } from 'multiformats'

export function Search() {
	const router = useRouter()
	const [imageId, setImageId] = useState('')
	const [isValid, setIsValid] = useState(false)

	useEffect(() => {
		try {
			CID.parse(imageId)
			setIsValid(true)
		} catch (error) {
			setIsValid(false)
		}
	}, [imageId])

	const handleKeyDown = (event: any) => {
		if (event.key === 'Enter') {
			handleNavigate()
		}
	}

	function handleClick() {
		handleNavigate()
	}

	function handleNavigate() {
		if (!imageId || !isValid) return
		router.push(`/images/${imageId}`)
	}

	return (
		<div className="flex gap-4">
			<Input
				type="search"
				placeholder="Search..."
				className="md:w-[100px] lg:w-[300px]"
				onChange={(e) => setImageId(e.target.value)}
				onKeyDown={handleKeyDown}
			/>

			<Button disabled={!imageId || !isValid} onClick={handleClick}>
				Search
			</Button>
		</div>
	)
}
