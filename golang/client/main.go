package main

import (
	"bufio"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"flag"
	"fmt"
	"github.com/joho/godotenv"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"log"
	"os"
	"proto/proto"
	"sync"
	"time"
)

var client proto.BroadcastClient
var wait *sync.WaitGroup

func init() {
	wait = &sync.WaitGroup{}
}

func connect(user *proto.User) error {
	var streamError error

	stream, err := client.CreateStream(context.Background(), &proto.Connect {
		User: 	user,
		Active: true,
	})

	if err != nil {
		return fmt.Errorf("connection failed: %v", err)
	}

	wait.Add(1)
	go func(stream proto.Broadcast_CreateStreamClient) {
		defer wait.Done()

		for {
			msg, err := stream.Recv()
			if err != nil {
				streamError = fmt.Errorf("error reading message: %v", err)
				break
			}

			fmt.Printf("%s: %s\n", msg.Id, msg.Content)
		}
	}(stream)

	return streamError
}

func main() {
	timestamp := time.Now()
	done := make(chan int)

	name := flag.String("name", "Anonymous", "Specify your username")
	flag.Parse()

	id := sha256.Sum256([]byte(fmt.Sprintf("%s%s", *name, timestamp.String())))

	err := godotenv.Load()
	if err != nil {
		log.Fatalf("error loading .env file")
	}

	HOST := fmt.Sprintf("localhost:%s", os.Getenv("PORT"))
	conn, err := grpc.Dial(HOST, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatalf("error connecting to service: %v\n", err)
	}

	client = proto.NewBroadcastClient(conn)
	user := &proto.User {
		Id: 	hex.EncodeToString(id[:]),
		Name: 	*name,
	}

	if err := connect(user); err != nil {
		log.Fatalf("error connecting to server: %v\n", err)
	}

	wait.Add(1)
	go func() {
		defer wait.Done()

		scanner := bufio.NewScanner(os.Stdin)
		for scanner.Scan() {
			msg := &proto.Message {
				Id: 		user.Id,
				Content: 	scanner.Text(),
				Timestamp: 	timestamp.String(),
			}

			if _, err := client.BroadcastMessage(context.Background(), msg); err != nil {
				fmt.Printf("error sending message: %v\n", err)
				break
			}
		}
	}()

	go func() {
		wait.Wait()
		close(done)
	}()

	<-done
}
